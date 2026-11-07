"""Map IL2CPP method names to the minified functions of an asm.js build (Unity 5.3 WebGL).

asm.js function pointers are indices into one table per signature (`var tp=[...]`, called as
`tp[i&mask](...)`). Il2CppDumper, run on the build's static memory (tools/re/asmjs_mem_to_wasm.py),
gives every method its pointer ("Address") and its wasm-style signature ("TypeSignature", e.g.
`vii`). Each `dynCall_<sig>` export names the table of that signature, so a method's function is
`table[sig][Address & mask]`.

Usage:
    python3 -I tools/re/asmjs_index.py <build.js> <script.json> <out index.json>

Output: {"functions": {asm name: [method names]}, "methods": {method name: asm name},
         "tables": {sig: table var}}
"""
import json
import re
import sys

js_path, script_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
js = open(js_path, encoding='latin-1').read()
script = json.load(open(script_path))

exports = dict(re.findall(r'dynCall_([a-z]+):([A-Za-z0-9_$]+)', js))
tables = {}
for name, body in re.findall(r'var ([A-Za-z0-9_$]+)=\[([A-Za-z0-9_$,]+)\];', js):
    tables[name] = body.split(',')

sig_table = {}
for sig, func in exports.items():
    m = re.search(r'function ' + re.escape(func) + r'\([^)]*\)\{[^}]*?([A-Za-z0-9_$]+)\[a&(\d+)\]\(', js)
    if m is None:
        print(f'no table call in dynCall_{sig} ({func})', file=sys.stderr)
        continue
    table, mask = m.group(1), int(m.group(2))
    assert table in tables and len(tables[table]) == mask + 1, (sig, table, mask)
    sig_table[sig] = table

FUNC_HEAD = re.compile(r'function ([A-Za-z0-9_$]+)\(([^)]*)\)\{')
heads = list(FUNC_HEAD.finditer(js))
spans = {}
for n, m in enumerate(heads):
    spans[m.group(1)] = (m, heads[n + 1].start() if n + 1 < len(heads) else len(js))


def asm_signature(name):
    """The signature of an asm.js function, from its parameter coercions (`a=a|0` int, `a=+a`
    double, `a=ea(a)` float) and its return statements."""
    m, end = spans[name]
    body = js[m.end():end]
    sig = ''
    for param in [p for p in m.group(2).split(',') if p != '']:
        coercion = re.search(r'(?<![A-Za-z0-9_$])' + re.escape(param) + r'=([^;]*);', body)
        text = coercion.group(1) if coercion is not None else ''
        if text.startswith('+'):
            sig += 'd'
        elif text.startswith('ea('):
            sig += 'f'
        else:
            sig += 'i'
    ret = 'v'
    for value in re.findall(r'return([^;}]*)', body):
        value = value.strip()
        if value == '':
            continue
        if value.startswith('+') or re.match(r'-?[0-9]+\.[0-9]', value) is not None:
            ret = 'd'
        elif value.startswith('ea('):
            ret = 'f'
        else:
            ret = 'i'
        break
    return ret + sig


def method_signature(method):
    """Il2CppDumper's TypeSignature, corrected for struct returns: a by-value struct (`Foo_o`, not
    `Foo_o*`) is returned through a hidden first pointer argument, so `iii` becomes `viii`."""
    signature = method.get('TypeSignature', '')
    returns = method.get('Signature', '').split(' ', 1)[0]
    if returns.endswith('_o') and signature.startswith('i'):
        return 'vi' + signature[1:]
    return signature


functions = {}
methods = {}
missing = 0
mismatches = []
for method in script['ScriptMethod']:
    sig = method_signature(method)
    table = sig_table.get(sig)
    if table is None:
        missing += 1
        continue
    entries = tables[table]
    func = entries[method['Address'] & (len(entries) - 1)]
    actual = asm_signature(func)
    if actual != sig:
        mismatches.append((method['Name'], sig, func, actual))
    functions.setdefault(func, []).append(method['Name'])
    methods[method['Name']] = func
json.dump({'functions': functions, 'methods': methods, 'tables': sig_table}, open(out_path, 'w'), indent=0)
for m in mismatches[:10]:
    print('signature mismatch', m, file=sys.stderr)
print(f'{len(mismatches)} signature mismatches', file=sys.stderr)
print(f'{len(methods)} methods -> {len(functions)} functions, {missing} without a table, {len(sig_table)} tables')

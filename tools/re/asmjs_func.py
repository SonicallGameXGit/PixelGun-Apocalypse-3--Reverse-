"""Print IL2CPP methods of an asm.js build (Unity 5.3 WebGL) as indented, annotated JavaScript.

Usage:
    python3 -I tools/re/asmjs_func.py <build.js> <index.json> <dump dir> <method substring | asm name>...

<index.json> comes from tools/re/asmjs_index.py; <dump dir> is the Il2CppDumper output (script.json,
dump.cs). Annotations:
- calls to known methods: `T5f/*ConnectMenu$$Start*/(...)`;
- heap views by their typed-array names (`HEAP32[x>>2]`), unless the function shadows the letter;
- loads of metadata slots: `HEAP32[239141]/*"Room "*/`, `/*ConnectMenu_TypeInfo*/`;
- `this` field offsets (first parameter of an instance method): `d+84/*.GamePlays*/`.
"""
import json
import re
import sys

HEAP_VIEWS = {
    'a': 'HEAP8', 'b': 'HEAP16', 'c': 'HEAP32', 'd': 'HEAPU8', 'e': 'HEAPU16', 'f': 'HEAPU32',
    'g': 'HEAPF32', 'h': 'HEAPF64', 'ea': 'fround', 'i': 'STACKTOP',
}
SHIFT = {'HEAP8': 0, 'HEAPU8': 0, 'HEAP16': 1, 'HEAPU16': 1, 'HEAP32': 2, 'HEAPU32': 2, 'HEAPF32': 2, 'HEAPF64': 3}
IDENT = r'[A-Za-z0-9_$]'


def class_fields(dump_path):
    """Instance field offsets per class name, from dump.cs."""
    fields = {}
    current = None
    for line in open(dump_path, encoding='utf-8'):
        m = re.match(r'(?:public |private |internal |protected |sealed |abstract |static )*(?:class|struct) (\S+)', line)
        if m is not None:
            current = fields.setdefault(m.group(1).split('<')[0], {})
            continue
        m = re.match(r'\t(?!.*\bstatic\b)(?!.*\bconst\b).*?(\S+); // 0x([0-9A-F]+)$', line)
        if m is not None and current is not None:
            current[int(m.group(2), 16)] = m.group(1)
    return fields


def pretty(code):
    out = []
    depth = 0
    line = ''
    paren = 0
    for ch in code:
        if ch == '(':
            paren += 1
        elif ch == ')':
            paren -= 1
        if ch == '{':
            out.append('    ' * depth + line.strip() + ' {')
            depth += 1
            line = ''
        elif ch == '}':
            if line.strip() != '':
                out.append('    ' * depth + line.strip())
            depth -= 1
            out.append('    ' * depth + '}')
            line = ''
        elif ch == ';' and paren == 0:
            out.append('    ' * depth + line.strip() + ';')
            line = ''
        else:
            line += ch
    if line.strip() != '':
        out.append(line.strip())
    return '\n'.join(out)


def main():
    js_path, index_path, dump_dir, *queries = sys.argv[1:]
    js = open(js_path, encoding='latin-1').read()
    index = json.load(open(index_path))
    script = json.load(open(dump_dir + '/script.json'))
    fields = class_fields(dump_dir + '/dump.cs')
    slots = {}
    for s in script['ScriptString']:
        slots[s['Address'] // 4] = json.dumps(s['Value'], ensure_ascii=False)
    for s in script['ScriptMetadata'] + script['ScriptMetadataMethod']:
        slots[s['Address'] // 4] = s['Name']
    names = index['functions']

    targets = []
    for q in queries:
        if q in names or re.fullmatch(IDENT + '+', q) is not None and ('function ' + q + '(') in js:
            targets.append(q)
        else:
            targets += sorted({f for m, f in index['methods'].items() if q in m})
    for func in targets:
        start = js.index('function ' + func + '(')
        nxt = re.compile(r'function ' + IDENT + r'+\(').search(js, start + 9)
        body = js[start:nxt.start() if nxt is not None else len(js)]
        head = re.match(r'function ' + IDENT + r'+\(([^)]*)\)\{((?:var [^;]*;)*)', body)
        params = [p for p in head.group(1).split(',') if p != '']
        local = set(params)
        for decl in re.findall(r'var ([^;]*);', body[:head.end()] + ';'):
            local |= {v.split('=')[0] for v in decl.split(',')}
        local |= set(re.findall(r'(?:^|[;{])\s*var\s+(' + IDENT + '+)', body))
        methods = names.get(func, [])
        owner = None
        if len(methods) > 0 and len(params) > 0:
            owner = fields.get(methods[0].split('$$')[0].split('.')[-1].split('<')[0])

        def heap(m):
            view = HEAP_VIEWS[m.group(1)]
            return view + '['

        for letter in HEAP_VIEWS:
            if letter in local or letter in ('ea', 'i'):
                continue
            body = re.sub(r'(?<!' + IDENT + r'|\.)(' + letter + r')\[', heap, body)
        if 'ea' not in local:
            body = re.sub(r'(?<!' + IDENT + r'|\.)ea\(', 'fround(', body)

        def slot(m):
            n = int(m.group(1))
            return m.group(0) + ('/*' + slots[n] + '*/' if n in slots else '')

        body = re.sub(r'HEAP32\[(\d+)\]', slot, body)

        def call(m):
            target = names.get(m.group(1))
            return m.group(1) + ('/*' + target[0] + '*/' if target else '') + '('

        body = re.sub(r'(?<!' + IDENT + r'|\.)(' + IDENT + r'+)\(', call, body)
        if owner is not None:
            this = params[0]

            def field(m):
                off = int(m.group(1))
                return m.group(0) + ('/*.' + owner[off] + '*/' if off in owner else '')

            body = re.sub(r'(?<!' + IDENT + r')' + re.escape(this) + r'\+(\d+)(?!\d)', field, body)
        print('// ' + func + ': ' + (', '.join(methods) if methods else '(no IL2CPP method)'))
        print(pretty(body))
        print()


main()

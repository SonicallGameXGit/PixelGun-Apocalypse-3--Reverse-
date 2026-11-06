"""Print selected functions from `wasm2c` output, annotated with IL2CPP method names.

Setup (once):
    wasm2c pga3-v18.wasm -o research/notes/wasm/pga3.c
    python3 -I tools/re/wasm_index.py pga3-v18.wasm script.json > research/notes/wasm/func_names.txt
Usage:
    python3 -I tools/re/wasm_func.py <pga3.c> <func_names.txt> <index | name-substring> ...

An argument that is a number selects that wasm function index; otherwise every method whose name
contains the substring is printed. Calls `w2c_<module>_f<N>` are shortened to `f<N>` and annotated
with `/*Method$$Name*/`; memory accesses are shortened to `i32[addr + off]` (`--raw` keeps the
original text); `--fold` additionally folds stack temporaries into expressions. Builds `<pga3.c>.idx` (byte offsets of each function) on first use.
"""
import json
import os
import re
import sys

DEF = re.compile(rb'^(?:static )?\S+ w2c_[A-Za-z0-9_]+?_f(\d+)\(w2c_[A-Za-z0-9_]+\* instance')
CALL = re.compile(r'w2c_[A-Za-z0-9]+?0x2Dv18_f(\d+)')
MEM = r'wasm_rt_local_memory_base, wasm_rt_local_memory_size, instance->w2c_env_memory, \(u64\)\((\w+)\)(?: \+ (\d+)u)?'
LOAD = re.compile(r'(\w+?)_load(\w*?)_default32\(' + MEM + r'\)')
STORE = re.compile(r'(\w+?)_store(\w*?)_default32\(' + MEM + r', ([^;]+)\);')
NOISE = ('FUNC_PROLOGUE;', 'FUNC_EPILOGUE;', '(void)wasm_rt_local_memory', 'wasm_rt_local_memory_size =',
         'uint8_t* const wasm_rt_local_memory_base', 'uint64_t wasm_rt_local_memory_size')


def simplify(line):
    """`i32_load_default32(<mem>, (u64)(x) + 52u)` -> `i32[x + 52]`, stores likewise."""
    def address(base, offset):
        return f'{base} + {offset}' if offset else base
    line = LOAD.sub(lambda m: f'{m.group(1)}{m.group(2)}[{address(m.group(3), m.group(4))}]', line)
    line = STORE.sub(lambda m: f'{m.group(1)}{m.group(2)}[{address(m.group(3), m.group(4))}] = {m.group(5)};', line)
    return line.replace('(instance, ', '(').replace('(instance)', '()')


TEMP = re.compile(r'\bvar_[ijfd]\d+\b')
TEMP_ASSIGN = re.compile(r'^(\s*)(var_[ijfd]\d+) (=|\+=|-=|\*=|/=|%=|&=|\|=|\^=|<<=|>>=) (.*);$')
LOCAL_ASSIGN = re.compile(r'^(\s*)(var_[lp]\d+) = (.*);$')
SIDE_EFFECT = re.compile(r'\bf\d+(?:/\*.*?\*/)?\(|CALL_INDIRECT|\[')
CALL_EXPR = re.compile(r'\bf\d+(?:/\*.*?\*/)?\(|CALL_INDIRECT')
ATOM = re.compile(r'-?[\w.$]+(?:\[[^\[\]]*\])?')


def paren(expr):
    return expr if ATOM.fullmatch(expr) else f'({expr})'


def fold(lines):
    """Fold wasm2c stack temporaries (var_i0, var_f1, ...) into expressions.

    Heuristic but order-preserving: values with loads/calls are emitted before any later side
    effect that does not consume them, and everything pending is emitted at block boundaries.
    """
    out = []
    pending = {}

    def substitute(text, consumed):
        def replace(match):
            name = match.group(0)
            if name in pending:
                consumed.add(name)
                return paren(pending[name])
            return name
        return TEMP.sub(replace, text)

    def flush(names, indent):
        for name in names:
            expr = pending.pop(name)
            if expr != name:
                out.append(f'{indent}{name} = {expr};')

    def flush_referencing(word, indent):
        pattern = re.compile(r'\b' + re.escape(word) + r'\b')
        flush([n for n, e in pending.items() if n != word and pattern.search(e)], indent)

    for line in lines:
        indent = line[:len(line) - len(line.lstrip())]
        m = TEMP_ASSIGN.match(line)
        if m:
            target, op, rhs = m.group(2), m.group(3), m.group(4)
            consumed = set()
            rhs = substitute(rhs, consumed)
            if op != '=':
                old = pending.get(target, target)
                rhs = f'{paren(old)} {op[:-1]} {paren(rhs)}'
            for name in consumed:
                pending.pop(name, None)
            pending.pop(target, None)
            flush_referencing(target, indent)
            if CALL_EXPR.search(rhs):
                flush([n for n, e in pending.items() if SIDE_EFFECT.search(e)], indent)
                out.append(f'{indent}{target} = {rhs};')
            else:
                pending[target] = rhs
            continue
        stripped = line.strip()
        boundary = stripped.endswith('{') or stripped.startswith('}') or stripped.endswith(':;') \
            or stripped.startswith('goto') or stripped.startswith('return') or stripped.startswith('switch')
        consumed = set()
        text = substitute(line, consumed)
        local = LOCAL_ASSIGN.match(line)
        rest = [n for n in pending if n not in consumed]
        # Block openings keep stack aliases of locals (still valid inside the block); closings,
        # labels (join points), goto and return must materialise every pending value.
        opening = stripped.endswith('{') and not stripped.startswith('}')
        if boundary and opening:
            flush([n for n in rest if not re.fullmatch(r'var_[lp]\d+', pending[n])], indent)
        elif boundary:
            flush(rest, indent)
        else:
            flush([n for n in rest if SIDE_EFFECT.search(pending[n])], indent)
        if local:
            flush_referencing(local.group(2), indent)
        out.append(text)
        for name in consumed:
            pending.pop(name, None)
        # local.tee: `var_l3 = var_f0;` keeps var_f0 on the stack, now equal to var_l3.
        tee = local and TEMP.fullmatch(local.group(3).strip())
        if tee:
            pending[local.group(3).strip()] = local.group(2)
    return out


def build_index(c_path):
    index = {}
    with open(c_path, 'rb') as f:
        offset = 0
        for line in f:
            if line.endswith(b'{\n'):
                match = DEF.match(line)
                if match:
                    index[int(match.group(1))] = offset
            offset += len(line)
    with open(c_path + '.idx', 'w') as out:
        json.dump(index, out)
    return index


def load_names(path):
    names = {}
    for line in open(path):
        parts = line.split(' ', 3)
        if len(parts) == 4 and parts[0].isdigit():
            names.setdefault(int(parts[0]), []).append(parts[3].strip())
    return names


def read_function(c_path, offset):
    lines = []
    with open(c_path, 'rb') as f:
        f.seek(offset)
        for raw in f:
            line = raw.decode('utf-8', 'replace').rstrip('\n')
            lines.append(line)
            if line == '}':
                break
    return lines


def main():
    args = sys.argv[1:]
    raw = '--raw' in args
    do_fold = '--fold' in args and not raw
    c_path, names_path, *queries = [a for a in args if a not in ('--raw', '--fold')]
    index_path = c_path + '.idx'
    if os.path.exists(index_path):
        index = {int(k): v for k, v in json.load(open(index_path)).items()}
    else:
        index = build_index(c_path)
    names = load_names(names_path)

    selected = []
    for query in queries:
        if query.isdigit():
            selected.append(int(query))
        else:
            selected.extend(f for f, ns in names.items() if any(query in n for n in ns))

    def annotate(match):
        func = int(match.group(1))
        label = names.get(func)
        return f'f{func}/*{" | ".join(label)}*/' if label else f'f{func}'

    for func in sorted(set(selected)):
        if func not in index:
            print(f'// f{func}: not found (imported or exported under another name)')
            continue
        print(f'// ===== f{func}: {" | ".join(names.get(func, ["?"]))}')
        lines = [CALL.sub(annotate, line if raw else simplify(line))
                 for line in read_function(c_path, index[func])
                 if raw or not any(n in line for n in NOISE)]
        if do_fold:
            lines = fold(lines)
        print('\n'.join(lines))
        print()


if __name__ == '__main__':
    main()

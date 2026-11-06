"""Map IL2CPP method names to wasm function indices for the WebGL build.

On IL2CPP WebGL (Unity 2018, Emscripten fastcomp), method "addresses" in Il2CppDumper's script.json
are function pointers into *per-signature* tables (`TypeSignature`, e.g. `vii`). Each exported
`dynCall_<sig>` computes the real table slot as `(pointer & mask) + base`; this reads mask/base from
those bodies, resolves the slot through the element segments and prints
`<func index> <pointer> <signature> <method name>`. Function indices count imported functions
first, i.e. the same index space WABT tools use (`wasm-decompile` names them `f_<index>`).

Usage: python3 -I wasm_index.py <module.wasm> <script.json> [name-substring ...]
"""
import json
import sys


def leb_u(data, pos):
    result = shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if byte < 0x80:
            return result, pos


def leb_s(data, pos):
    result = shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if byte < 0x80:
            if byte & 0x40:
                result -= 1 << shift
            return result, pos


def const_expr(data, pos, globals_init):
    opcode = data[pos]
    pos += 1
    if opcode == 0x41:  # i32.const
        value, pos = leb_s(data, pos)
    elif opcode == 0x23:  # global.get (imported base, e.g. __table_base)
        index, pos = leb_u(data, pos)
        value = globals_init.get(index, 0)
    else:
        raise ValueError(f'unsupported init opcode {opcode:#x}')
    assert data[pos] == 0x0B
    return value, pos + 1


def read_module(path):
    data = open(path, 'rb').read()
    assert data[:8] == b'\0asm\x01\0\0\0'
    pos = 8
    imported_funcs = 0
    table = {}
    exports = {}
    code = None
    types = []
    func_types = []
    while pos < len(data):
        section = data[pos]
        size, pos = leb_u(data, pos + 1)
        end = pos + size
        if section == 2:  # imports
            count, p = leb_u(data, pos)
            for _ in range(count):
                for _ in range(2):
                    length, p = leb_u(data, p)
                    p += length
                kind = data[p]
                p += 1
                if kind == 0:
                    type_index, p = leb_u(data, p)
                    imported_funcs += 1
                    func_types.append(type_index)
                elif kind == 1:
                    p += 1
                    flags, p = leb_u(data, p)
                    _, p = leb_u(data, p)
                    if flags & 1:
                        _, p = leb_u(data, p)
                elif kind == 2:
                    flags, p = leb_u(data, p)
                    _, p = leb_u(data, p)
                    if flags & 1:
                        _, p = leb_u(data, p)
                elif kind == 3:
                    p += 2
        elif section == 1:  # types -> signature letters (Emscripten dynCall convention)
            letters = {0x7F: 'i', 0x7E: 'j', 0x7D: 'f', 0x7C: 'd'}
            count, p = leb_u(data, pos)
            for _ in range(count):
                assert data[p] == 0x60
                n, p = leb_u(data, p + 1)
                params = ''.join(letters[b] for b in data[p:p + n])
                p += n
                n, p = leb_u(data, p)
                result = ''.join(letters[b] for b in data[p:p + n]) or 'v'
                p += n
                types.append(result + params)
        elif section == 3:  # function -> type index
            count, p = leb_u(data, pos)
            for _ in range(count):
                index, p = leb_u(data, p)
                func_types.append(index)
        elif section == 7:  # exports
            count, p = leb_u(data, pos)
            for _ in range(count):
                length, p = leb_u(data, p)
                name = data[p:p + length].decode()
                p += length
                kind = data[p]
                index, p = leb_u(data, p + 1)
                if kind == 0:
                    exports[name] = index
        elif section == 10:  # code
            code = pos
        elif section == 9:  # elements
            count, p = leb_u(data, pos)
            for _ in range(count):
                flags, p = leb_u(data, p)
                assert flags == 0, f'element segment flags {flags} not supported'
                offset, p = const_expr(data, p, {})
                n, p = leb_u(data, p)
                for i in range(n):
                    func, p = leb_u(data, p)
                    table[offset + i] = func
        pos = end

    bodies = {}
    count, p = leb_u(data, code)
    for i in range(count):
        size, p = leb_u(data, p)
        bodies[imported_funcs + i] = (p, size)
        p += size

    # dynCall_<sig>: ... local.get 0; i32.const MASK; i32.and; i32.const BASE; i32.add; call_indirect
    signatures = {}
    for name, func in exports.items():
        if not name.startswith('dynCall_'):
            continue
        start, size = bodies[func]
        body = data[start:start + size]
        at = body.find(b'\x71\x41')
        if at < 0:
            continue
        mask_at = body.rfind(b'\x41', 0, at)
        mask, _ = leb_s(body, mask_at + 1)
        base, after = leb_s(body, at + 2)
        if body[after] != 0x6A:
            continue
        signatures[name[len('dynCall_'):]] = (mask, base)
    func_sigs = [types[t] for t in func_types]  # absolute function index -> signature
    return imported_funcs, table, signatures, func_sigs


def method_signature(method):
    """Il2CppDumper's TypeSignature, corrected for struct returns: a by-value struct (`Foo_o`, not
    `Foo_o*`) is returned through a hidden first pointer argument, so `iii` becomes `viii`."""
    signature = method.get('TypeSignature', '')
    returns = method.get('Signature', '').split(' ', 1)[0]
    if returns.endswith('_o') and signature.startswith('i'):
        return 'vi' + signature[1:]
    return signature


def main():
    wasm_path, script_path, *filters = sys.argv[1:]
    imported, table, signatures, func_sigs = read_module(wasm_path)
    methods = json.load(open(script_path))['ScriptMethod']
    print(f'# {imported} imported functions, {len(table)} table entries, '
          f'{len(signatures)} signatures', file=sys.stderr)
    mismatches = 0
    for method in methods:
        if filters and not any(f in method['Name'] for f in filters):
            continue
        signature = method_signature(method)
        func = None
        if signature in signatures:
            mask, base = signatures[signature]
            func = table.get((method['Address'] & mask) + base)
        if func is not None and func_sigs[func] != signature:
            mismatches += 1
            func = None
        print(f'{func if func is not None else "?"} {method["Address"]} {signature} {method["Name"]}')
    print(f'# {mismatches} unresolved because the table entry has a different wasm type', file=sys.stderr)


if __name__ == '__main__':
    main()

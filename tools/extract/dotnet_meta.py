"""Minimal ECMA-335 metadata reader: TypeDef and Field tables of a .NET assembly (e.g. Il2CppDumper DummyDll).

Only what the MonoBehaviour decoder needs: per type (namespace, name) its fields with flags, notably
FIELD_ATTRIBUTE_NOT_SERIALIZED (0x0080), which Il2CppDumper's dump.cs does not print.
"""
import struct

NOT_SERIALIZED = 0x0080
STATIC = 0x0010
LITERAL = 0x0040


def _rva_to_offset(sections, rva):
    for va, vsize, raw_ptr, raw_size in sections:
        if va <= rva < va + max(vsize, raw_size):
            return raw_ptr + (rva - va)
    raise ValueError(f'RVA {rva:#x} not in any section')


def read_types(path):
    data = open(path, 'rb').read()
    pe = struct.unpack_from('<I', data, 0x3C)[0]
    assert data[pe:pe + 4] == b'PE\0\0'
    num_sections = struct.unpack_from('<H', data, pe + 6)[0]
    opt_size = struct.unpack_from('<H', data, pe + 20)[0]
    opt = pe + 24
    magic = struct.unpack_from('<H', data, opt)[0]
    dd = opt + (96 if magic == 0x10B else 112)
    cli_rva = struct.unpack_from('<I', data, dd + 14 * 8)[0]
    sec = opt + opt_size
    sections = []
    for i in range(num_sections):
        vsize, va, raw_size, raw_ptr = struct.unpack_from('<IIII', data, sec + i * 40 + 8)
        sections.append((va, vsize, raw_ptr, raw_size))

    cli = _rva_to_offset(sections, cli_rva)
    meta_rva = struct.unpack_from('<I', data, cli + 8)[0]
    root = _rva_to_offset(sections, meta_rva)
    assert struct.unpack_from('<I', data, root)[0] == 0x424A5342
    ver_len = struct.unpack_from('<I', data, root + 12)[0]
    p = root + 16 + ver_len + 2
    n_streams = struct.unpack_from('<H', data, p)[0]
    p += 2
    streams = {}
    for _ in range(n_streams):
        off, size = struct.unpack_from('<II', data, p)
        p += 8
        end = data.index(b'\0', p)
        name = data[p:end].decode()
        p = (end + 4) & ~3
        streams[name] = (root + off, size)

    strings_off = streams['#Strings'][0]
    tables_off = streams.get('#~', streams.get('#-'))[0]
    heap_sizes = data[tables_off + 6]
    valid = struct.unpack_from('<Q', data, tables_off + 8)[0]
    p = tables_off + 24
    rows = {}
    for t in range(64):
        if valid >> t & 1:
            rows[t] = struct.unpack_from('<I', data, p)[0]
            p += 4
    assert 3 not in rows, 'FieldPtr table not supported'

    str_size = 4 if heap_sizes & 1 else 2
    guid_size = 4 if heap_sizes & 2 else 2
    blob_size = 4 if heap_sizes & 4 else 2

    def idx_size(table):
        return 4 if rows.get(table, 0) > 0xFFFF else 2

    def coded_size(tables, bits):
        return 4 if max(rows.get(t, 0) for t in tables) >= (1 << (16 - bits)) else 2

    def read_idx(off, size):
        return struct.unpack_from('<I' if size == 4 else '<H', data, off)[0]

    def get_string(index):
        start = strings_off + index
        return data[start:data.index(b'\0', start)].decode('utf-8', 'replace')

    module_row = 2 + str_size + 3 * guid_size
    typeref_row = coded_size([0, 0x1A, 0x23, 1], 2) + 2 * str_size
    tdor = coded_size([2, 1, 0x1B], 2)
    typedef_row = 4 + 2 * str_size + tdor + idx_size(4) + idx_size(6)
    field_row = 2 + str_size + blob_size

    typedef_off = p + rows.get(0, 0) * module_row + rows.get(1, 0) * typeref_row
    field_off = typedef_off + rows.get(2, 0) * typedef_row

    fields = []
    for i in range(rows.get(4, 0)):
        o = field_off + i * field_row
        flags = struct.unpack_from('<H', data, o)[0]
        fields.append((get_string(read_idx(o + 2, str_size)), flags))

    types = []
    n_types = rows.get(2, 0)
    starts = []
    for i in range(n_types):
        o = typedef_off + i * typedef_row
        name = get_string(read_idx(o + 4, str_size))
        namespace = get_string(read_idx(o + 4 + str_size, str_size))
        field_list = read_idx(o + 4 + 2 * str_size + tdor, idx_size(4))
        starts.append(field_list)
        types.append({'namespace': namespace, 'name': name})
    for i, t in enumerate(types):
        start = starts[i] - 1
        end = (starts[i + 1] - 1) if i + 1 < n_types else len(fields)
        t['fields'] = fields[start:end]
    return types

import sys, struct, os
src, out = sys.argv[1], sys.argv[2]
d = open(src, 'rb').read()
assert d[:16] == b'UnityWebData1.0\0'
hdr = struct.unpack_from('<I', d, 16)[0]
p = 20
while p < hdr:
    off, size, nlen = struct.unpack_from('<III', d, p); p += 12
    name = d[p:p+nlen].decode(); p += nlen
    dst = os.path.normpath(os.path.join(out, name))
    assert dst.startswith(os.path.abspath(out))
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    open(dst, 'wb').write(d[off:off+size])
    print(f'{size:>10}  {name}')

"""Unpack a Unity 5.3 WebGL `.data` file (an Emscripten file package) into its files.

The file table is not in the data file: Unity 5.3's UnityLoader.js embeds the packager's loader,
which lists every file as `new o(start, end, ...).open("GET", "/path")`.

Usage:
    python3 -I tools/extract/unpack_emscripten_package.py <UnityLoader.js> <file.data> <out dir>
"""
import os
import re
import sys

loader, src, out = sys.argv[1], sys.argv[2], sys.argv[3]
table = open(loader, encoding='utf-8').read()
data = open(src, 'rb').read()
entries = re.findall(r'new \w+\((\d+),(\d+),\d+,\d+\)\.open\("GET","([^"]+)"\)', table)
assert len(entries) > 0, 'no file table in ' + loader
out = os.path.abspath(out)
for start, end, name in entries:
    start, end = int(start), int(end)
    assert 0 <= start <= end <= len(data), (name, start, end)
    dst = os.path.normpath(os.path.join(out, name.lstrip('/')))
    assert dst.startswith(out + os.sep), name
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    with open(dst, 'wb') as f:
        f.write(data[start:end])
    print(f'{end - start:>10}  {name}')
print(f'{len(entries)} files, {max(int(e) for _, e, _ in entries)} of {len(data)} bytes')

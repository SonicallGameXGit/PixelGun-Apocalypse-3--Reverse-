"""Wrap an asm.js build's static memory (`*.js.mem`) in a minimal wasm module for Il2CppDumper.

Unity 5.3 WebGL builds are asm.js, which Il2CppDumper cannot read. Its wasm loader only uses the
module's data section, as the memory image in which it searches the IL2CPP code and metadata
registrations. A module with one data segment holding the `.mem` file at `Runtime.GLOBAL_BASE` gives
it the same memory. Method addresses in its output are then asm.js function-table indices
(see tools/re/asmjs_index.py).

Usage:
    python3 -I tools/re/asmjs_mem_to_wasm.py <file.js.mem> <out.wasm> [global base, default 8]
"""
import sys


def uleb(value):
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value != 0:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def sleb(value):
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        done = (value == 0 and byte & 0x40 == 0) or (value == -1 and byte & 0x40 != 0)
        out.append(byte if done else byte | 0x80)
        if done:
            return bytes(out)


mem = open(sys.argv[1], 'rb').read()
base = int(sys.argv[3]) if len(sys.argv) > 3 else 8
segment = uleb(0) + b'\x41' + sleb(base) + b'\x0b' + uleb(len(mem)) + mem
section = uleb(1) + segment
module = b'\0asm' + (1).to_bytes(4, 'little') + bytes([11]) + uleb(len(section)) + section
open(sys.argv[2], 'wb').write(module)
print(f'{len(mem)} bytes of static memory at {base} -> {sys.argv[2]}')

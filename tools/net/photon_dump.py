"""Decode a WebSocket capture of a Photon (Protocol16, GpBinaryV16) session into readable text.

Input: the JSON array written by the in-page WebSocket hook (see tools/net/README.md):
    [{"t": ms, "dir": "open"|"in"|"out"|"close", "url"/"ws": str, "d": base64 payload}, ...]
Usage:
    python3 -I tools/net/photon_dump.py <capture.json> [--skip-pings] [--redact]

`--redact` replaces the auth secret (parameter 221) and user id (225) with placeholders, so the output
can be quoted in docs.
"""
import base64
import json
import struct
import sys

MESSAGE_TYPES = {
    1: 'InitResponse', 2: 'OpRequest', 3: 'OpResponse', 4: 'Event', 6: 'InternalOpRequest',
    7: 'InternalOpResponse', 8: 'Message', 9: 'RawMessage',
}


class Custom:
    def __init__(self, code, data):
        self.code = code
        self.data = data

    def __repr__(self):
        if self.code == ord('V') and len(self.data) == 12:
            return 'V3(%g, %g, %g)' % struct.unpack('>3f', self.data)
        if self.code == ord('W') and len(self.data) == 8:
            return 'V2(%g, %g)' % struct.unpack('>2f', self.data)
        if self.code == ord('Q') and len(self.data) == 16:
            return 'Q(w=%g, %g, %g, %g)' % struct.unpack('>4f', self.data)
        if self.code == ord('P') and len(self.data) == 4:
            return 'Player(%d)' % struct.unpack('>i', self.data)
        return f'Custom({chr(self.code)!r}, {self.data.hex()})'


class Reader:
    def __init__(self, data):
        self.data = data
        self.pos = 0

    def take(self, n):
        if self.pos + n > len(self.data):
            raise ValueError(f'read past end at {self.pos} (+{n}) of {len(self.data)}')
        chunk = self.data[self.pos:self.pos + n]
        self.pos += n
        return chunk

    def byte(self):
        return self.take(1)[0]

    def short(self):
        return struct.unpack('>h', self.take(2))[0]

    def int(self):
        return struct.unpack('>i', self.take(4))[0]

    def string(self):
        return self.take(struct.unpack('>H', self.take(2))[0]).decode('utf-8', 'replace')

    def typed(self):
        return self.value(self.byte())

    def table(self):
        return {self.byte(): self.typed() for _ in range(self.short())}

    def value(self, t):
        if t in (0, 42):
            return None
        if t == 68:  # 'D' Dictionary
            key_type, value_type = self.byte(), self.byte()
            out = {}
            for _ in range(self.short()):
                key = self.typed() if key_type in (0, 42) else self.value(key_type)
                out[freeze(key)] = self.typed() if value_type in (0, 42) else self.value(value_type)
            return out
        if t == 97:  # 'a' string[]
            return [self.string() for _ in range(self.short())]
        if t == 98:  # 'b' byte
            return self.byte()
        if t == 99:  # 'c' custom
            code = self.byte()
            return Custom(code, self.take(self.short()))
        if t == 100:  # 'd' double
            return struct.unpack('>d', self.take(8))[0]
        if t == 101:  # 'e' EventData
            return ('EventData', self.byte(), self.table())
        if t == 102:  # 'f' float
            return struct.unpack('>f', self.take(4))[0]
        if t == 104:  # 'h' Hashtable
            out = {}
            for _ in range(self.short()):
                key = self.typed()
                out[freeze(key)] = self.typed()
            return out
        if t == 105:  # 'i' int
            return self.int()
        if t == 107:  # 'k' short
            return self.short()
        if t == 108:  # 'l' long
            return struct.unpack('>q', self.take(8))[0]
        if t == 110:  # 'n' int[]
            return [self.int() for _ in range(self.int())]
        if t == 111:  # 'o' bool
            return self.byte() != 0
        if t == 115:  # 's' string
            return self.string()
        if t == 120:  # 'x' byte[]
            return self.take(self.int())
        if t == 121:  # 'y' typed array
            n = self.short()
            element = self.byte()
            if element == 99:
                code = self.byte()
                return [Custom(code, self.take(self.short())) for _ in range(n)]
            return [self.value(element) for _ in range(n)]
        if t == 122:  # 'z' object[]
            return [self.typed() for _ in range(self.short())]
        raise ValueError(f'unknown Protocol16 type {t} ({chr(t)!r}) at {self.pos - 1}')


def freeze(key):
    return tuple(key) if isinstance(key, list) else key


def decode(payload):
    """Returns (kind, fields) for one WebSocket binary message."""
    if len(payload) < 2 or payload[0] != 0xF3:
        return 'Raw', {'hex': payload.hex()}
    kind = payload[1] & 0x7F
    if payload[1] & 0x80:
        return 'Encrypted', {'type': kind, 'hex': payload.hex()}
    r = Reader(payload[2:])
    name = MESSAGE_TYPES.get(kind, f'Type{kind}')
    if kind == 1:
        return name, {'rest': r.data.hex()}
    if kind in (2, 6):
        return name, {'op': r.byte(), 'params': r.table()}
    if kind in (3, 7):
        op = r.byte()
        code = r.short()
        debug = r.typed()
        return name, {'op': op, 'return': code, 'debug': debug, 'params': r.table()}
    if kind == 4:
        return name, {'code': r.byte(), 'params': r.table()}
    return name, {'hex': r.data.hex()}


def redact(value):
    if isinstance(value, dict):
        out = {}
        for k, v in value.items():
            if k == 221 and isinstance(v, str):
                out[k] = '<secret>'
            elif k == 225 and isinstance(v, str):
                out[k] = '<user-id>'
            else:
                out[k] = redact(v)
        return out
    return value


def main():
    args = sys.argv[1:]
    skip_pings = '--skip-pings' in args
    do_redact = '--redact' in args
    path = [a for a in args if not a.startswith('--')][0]
    log = json.load(open(path))
    t0 = log[0]['t']
    for entry in log:
        url = entry.get('url') or entry.get('ws') or ''
        host = url.split('//')[-1].split('.')[0] + ('/game' if url.endswith('/game') else '')
        stamp = f'{(entry["t"] - t0) / 1000:9.3f} {entry["dir"]:5} {host:12}'
        if entry['dir'] in ('open', 'close'):
            print(stamp, entry.get('protocols') or entry.get('code'))
            continue
        payload = base64.b64decode(entry['d'])
        try:
            kind, fields = decode(payload)
        except ValueError as error:
            print(stamp, 'DECODE ERROR', error, payload.hex())
            continue
        if skip_pings and kind in ('InternalOpRequest', 'InternalOpResponse'):
            continue
        if do_redact:
            fields = redact(fields)
        print(stamp, kind, fields)


if __name__ == '__main__':
    main()

"""Select a sanitized sample of recorded Photon frames for the codec golden test.

Usage:
    python3 -I tools/net/make_fixture.py <capture.json> > docs/fixtures/photon-crazygames-frames.txt

Keeps up to 4 frames of every (direction, message type, op/event code) and drops every frame that
carries the auth secret (parameter 221) or the user id (225), so nothing account-related reaches
the repository. Each output line is `<dir> <hex>`.
"""
import base64
import collections
import json
import sys

sys.path.insert(0, __file__.rsplit('/', 1)[0])
import photon_dump  # noqa: E402


def has_secret(fields):
    params = fields.get('params', {})
    return 221 in params or 225 in params


def main():
    log = json.load(open(sys.argv[1]))
    seen = collections.Counter()
    print('# Recorded Photon frames (CrazyGames build, 2026-11-07), sanitized by tools/net/make_fixture.py.')
    for entry in log:
        if entry['dir'] not in ('in', 'out'):
            continue
        payload = base64.b64decode(entry['d'])
        kind, fields = photon_dump.decode(payload)
        if has_secret(fields):
            continue
        key = (entry['dir'], kind, fields.get('op', fields.get('code')))
        if seen[key] >= 4:
            continue
        seen[key] += 1
        print(entry['dir'], payload.hex())


if __name__ == '__main__':
    main()

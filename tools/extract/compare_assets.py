"""Compare two published asset trees (e.g. Build-7 `assets/` and the CrazyGames staging tree).

Usage: venv/bin/python -I compare_assets.py <assets-a> <assets-b> <out.json>

Files are matched by content, not by name (`__<path_id>` suffixes and some names differ between
builds): textures/sprites by decoded RGBA pixels, everything else by bytes. For files present on
one side only, the closest same-name (suffix stripped) file on the other side is reported, with
the mean absolute RGBA difference for images.
"""
import hashlib
import json
import os
import re
import sys

from PIL import Image

a_dir, b_dir, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
IMAGE_TYPES = {'Texture2D', 'Sprite'}


def base_name(path):
    stem, ext = os.path.splitext(os.path.basename(path))
    return re.sub(r'__\d+$', '', stem) + ext


def content_hash(root, entry):
    path = os.path.join(root, entry['path'])
    if entry['type'] in IMAGE_TYPES:
        img = Image.open(path).convert('RGBA')
        return hashlib.sha256(repr(img.size).encode() + img.tobytes()).hexdigest()
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


def image_difference(path_a, path_b):
    img_a = Image.open(path_a).convert('RGBA')
    img_b = Image.open(path_b).convert('RGBA')
    sizes = [img_a.size, img_b.size]
    if img_a.size != img_b.size:
        img_b = img_b.resize(img_a.size, Image.BILINEAR)
    data_a, data_b = img_a.tobytes(), img_b.tobytes()
    total = sum(abs(x - y) for x, y in zip(data_a, data_b))
    return {'sizes': sizes, 'mean_abs_diff': round(total / len(data_a), 3)}


def load(root):
    manifest = json.load(open(os.path.join(root, 'manifest.json')))
    for entry in manifest:
        entry['hash'] = content_hash(root, entry)
    return manifest


side_a, side_b = load(a_dir), load(b_dir)
report = {}
for kind in sorted({e['type'] for e in side_a + side_b}):
    a = [e for e in side_a if e['type'] == kind]
    b = [e for e in side_b if e['type'] == kind]
    hashes_a = {e['hash'] for e in a}
    hashes_b = {e['hash'] for e in b}
    by_name_a, by_name_b = {}, {}
    for e in a:
        by_name_a.setdefault(base_name(e['path']), []).append(e)
    for e in b:
        by_name_b.setdefault(base_name(e['path']), []).append(e)
    same = sum(1 for e in a if e['hash'] in hashes_b)
    renamed = sorted([e['path'], next(f['path'] for f in b if f['hash'] == e['hash'])]
                     for e in a if e['hash'] in hashes_b
                     and base_name(e['path']) != base_name(next(f['path'] for f in b if f['hash'] == e['hash'])))
    changed = []
    only_a = []
    for e in a:
        if e['hash'] in hashes_b:
            continue
        candidates = [f for f in by_name_b.get(base_name(e['path']), []) if f['hash'] not in hashes_a]
        if candidates:
            f = candidates[0]
            item = {'a': e['path'], 'b': f['path']}
            if kind in IMAGE_TYPES:
                item.update(image_difference(os.path.join(a_dir, e['path']), os.path.join(b_dir, f['path'])))
            changed.append(item)
        else:
            only_a.append(e['path'])
    matched_b = {c['b'] for c in changed}
    only_b = [e['path'] for e in b if e['hash'] not in hashes_a and e['path'] not in matched_b]
    report[kind] = {
        'count_a': len(a), 'count_b': len(b), 'identical': same, 'identical_renamed': renamed,
        'changed': changed, 'only_a': sorted(only_a), 'only_b': sorted(only_b),
    }
    print(f'{kind:<10} a {len(a):>4}  b {len(b):>4}  identical {same:>4} (renamed {len(renamed)})  '
          f'changed {len(changed):>4}  only a {len(only_a):>4}  only b {len(only_b):>4}')

with open(out_path, 'w') as f:
    json.dump(report, f, indent=4)
    f.write('\n')

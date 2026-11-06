"""Copy curated, game-owned assets from the raw extraction into the committed assets/ tree.

Usage: python3 -I publish_assets.py <extracted-dir> <assets-dir>

- Skips Unity's built-in resources (unity_default_resources, unity_builtin_extra): those are
  Unity's, not the game's. Exception: the built-in "Arial" font object, whose embedded data is
  Liberation Sans 2.00.1 (SIL Open Font License 1.1, see assets/fonts/OFL-LiberationSans.txt);
  the game's IMGUI skins draw all their text with it.
- Writes <assets-dir>/manifest.json mapping each published file to its original Unity object.
"""
import json
import os
import shutil
import sys

src, dst = sys.argv[1], sys.argv[2]

UNITY_BUILTIN = {'unity_default_resources', 'unity_builtin_extra'}
# Built-in objects that are freely licensed and needed by the client: (file, path_id).
BUILTIN_ALLOWED = {('unity_default_resources', 10102)}
TARGET_DIR = {
    'Texture2D': 'textures',
    'Sprite': 'ui/sprites',
    'AudioClip': 'audio',
    'Mesh': 'models',
    'Font': 'fonts',
}

inventory = json.load(open(os.path.join(src, 'inventory.json')))

for sub in set(TARGET_DIR.values()):
    path = os.path.join(dst, sub)
    if os.path.isdir(path):
        for name in os.listdir(path):
            full = os.path.join(path, name)
            if os.path.isfile(full) and name != '.gitkeep' and not name.startswith('OFL-'):
                os.remove(full)

manifest = []
for entry in inventory:
    target = TARGET_DIR.get(entry['type'])
    if target is None or entry.get('exported') is None:
        continue
    if entry['file'] in UNITY_BUILTIN and (entry['file'], entry['path_id']) not in BUILTIN_ALLOWED:
        continue
    file_name = os.path.basename(entry['exported'])
    out_rel = f'{target}/{file_name}'
    os.makedirs(os.path.join(dst, target), exist_ok=True)
    shutil.copyfile(os.path.join(src, entry['exported']), os.path.join(dst, out_rel))
    item = {
        'path': out_rel,
        'type': entry['type'],
        'name': entry.get('name'),
        'source': { 'file': entry['file'], 'path_id': entry['path_id'] },
    }
    if entry.get('container') is not None:
        item['container'] = entry['container']
    for key in ('size', 'audio'):
        if entry.get(key) is not None:
            item[key] = entry[key]
    manifest.append(item)

manifest.sort(key=lambda m: m['path'])
with open(os.path.join(dst, 'manifest.json'), 'w') as f:
    json.dump(manifest, f, indent=4)
    f.write('\n')

counts = {}
for m in manifest:
    counts[m['type']] = counts.get(m['type'], 0) + 1
print(counts, 'total', len(manifest))

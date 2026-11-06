"""Exploratory asset extractor for the PGA3 Unity build (UnityPy).

Usage: venv/bin/python -I extract_assets.py <unpacked-dir> <out-dir>

Writes:
  <out>/inventory.json         every object: file, path_id, type, name, container path
  <out>/Texture2D/*.png        decoded textures
  <out>/Sprite/*.png           sprites
  <out>/AudioClip/*.<ext>      raw audio payload (extension from magic bytes)
  <out>/TextAsset/*            raw text assets
  <out>/Font/*                 raw font data
  <out>/Mesh/*.obj             meshes
  <out>/MonoBehaviour/*.bin    raw serialized script data (typetrees are stripped)
  <out>/errors.json            per-object export failures
"""
import json
import os
import re
import sys
import traceback

import UnityPy

src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)

env = UnityPy.load(src)

container_by_pid = {}
for path, obj in env.container.items():
    container_by_pid[(obj.assets_file.name, obj.path_id)] = path

used_names = {}


def safe_name(kind, name, pid):
    base = re.sub(r'[^A-Za-z0-9._ -]+', '_', name or '').strip() or f'unnamed_{pid}'
    key = (kind, base.lower())
    count = used_names.get(key, 0)
    used_names[key] = count + 1
    return base if count == 0 else f'{base}__{pid}'


def audio_ext(data):
    if data[:4] == b'OggS':
        return 'ogg'
    if data[:4] == b'RIFF':
        return 'wav'
    if data[:4] == b'FSB5':
        return 'fsb'
    if data[4:8] == b'ftyp':
        return 'm4a'
    if data[:3] == b'ID3' or data[:2] in (b'\xff\xfb', b'\xff\xf3', b'\xff\xf2'):
        return 'mp3'
    return 'bin'


import struct

files_by_name = {}
for obj in env.objects:
    files_by_name.setdefault(obj.assets_file.name, obj.assets_file)

script_names = {}
for obj in env.objects:
    if obj.type.name == 'MonoScript':
        ms = obj.read()
        script_names[(obj.assets_file.name, obj.path_id)] = f'{ms.m_Namespace}.{ms.m_ClassName}'.lstrip('.')


def resolve_file(assets_file, file_id):
    if file_id == 0:
        return assets_file.name
    ext = assets_file.externals[file_id - 1]
    return os.path.basename(ext.path)


def parse_mono_header(obj, raw):
    # m_GameObject PPtr, m_Enabled (u8, aligned to 4), m_Script PPtr, m_Name (aligned string)
    p = 0
    go_file, go_pid = struct.unpack_from('<iq', raw, p); p += 12
    enabled = raw[p]; p += 4
    sc_file, sc_pid = struct.unpack_from('<iq', raw, p); p += 12
    nlen = struct.unpack_from('<I', raw, p)[0]; p += 4
    name = raw[p:p + nlen].decode('utf-8', 'replace'); p += nlen
    p = (p + 3) & ~3
    script = script_names.get((resolve_file(obj.assets_file, sc_file), sc_pid))
    return {'game_object': [go_file, go_pid], 'enabled': enabled, 'script': script,
            'name': name, 'fields_offset': p}


resource_cache = {}


def read_resource(source, offset, size):
    base = os.path.basename(source)
    if base not in resource_cache:
        resource_cache[base] = open(os.path.join(src, base), 'rb').read()
    return resource_cache[base][offset:offset + size]


inventory = []
errors = []
for obj in env.objects:
    tname = obj.type.name
    entry = {
        'file': obj.assets_file.name,
        'path_id': obj.path_id,
        'type': tname,
        'container': container_by_pid.get((obj.assets_file.name, obj.path_id)),
    }
    try:
        if tname == 'MonoBehaviour':
            raw = obj.get_raw_data()
            hdr = parse_mono_header(obj, raw)
            entry.update(name=hdr['name'], script=hdr['script'], game_object=hdr['game_object'],
                         fields_offset=hdr['fields_offset'])
            kind_dir = os.path.join(out, 'MonoBehaviour', hdr['script'] or 'unknown')
            os.makedirs(kind_dir, exist_ok=True)
            fn = safe_name(f'MB:{hdr["script"]}', f'{hdr["name"] or "go"}', obj.path_id) + '.bin'
            fn = f'{obj.assets_file.name}__{obj.path_id}__{fn}'
            open(os.path.join(kind_dir, fn), 'wb').write(raw)
            entry['exported'] = f'MonoBehaviour/{hdr["script"] or "unknown"}/{fn}'
        elif tname in ('Texture2D', 'Sprite', 'AudioClip', 'TextAsset', 'Font', 'Mesh',
                     'MonoScript', 'GameObject', 'Material', 'Shader',
                     'AnimationClip', 'Cubemap'):
            data = obj.read()
            name = getattr(data, 'm_Name', None) or getattr(data, 'name', None) or ''
            entry['name'] = name
            kind_dir = os.path.join(out, tname)
            if tname == 'Texture2D' and getattr(data, 'm_Width', 1) == 0:
                entry['skipped'] = 'empty runtime-generated texture (dynamic font atlas)'
            elif tname in ('Texture2D', 'Sprite'):
                os.makedirs(kind_dir, exist_ok=True)
                img = data.image
                entry['size'] = list(img.size)
                fn = safe_name(tname, name, obj.path_id) + '.png'
                img.save(os.path.join(kind_dir, fn))
                entry['exported'] = f'{tname}/{fn}'
            elif tname == 'AudioClip':
                os.makedirs(kind_dir, exist_ok=True)
                raw = data.m_AudioData
                if isinstance(raw, list):
                    raw = bytes(raw)
                res = data.m_Resource
                if not raw and res is not None and res.m_Size:
                    raw = read_resource(res.m_Source, res.m_Offset, res.m_Size)
                    entry['resource'] = [res.m_Source, res.m_Offset, res.m_Size]
                if raw:
                    fn = safe_name(tname, name, obj.path_id) + '.' + audio_ext(raw[:16])
                    open(os.path.join(kind_dir, fn), 'wb').write(raw)
                    entry['exported'] = f'{tname}/{fn}'
                entry['audio'] = {
                    'channels': getattr(data, 'm_Channels', None),
                    'frequency': getattr(data, 'm_Frequency', None),
                    'length': getattr(data, 'm_Length', None),
                    'compression': str(getattr(data, 'm_CompressionFormat', None)),
                }
            elif tname == 'TextAsset':
                os.makedirs(kind_dir, exist_ok=True)
                script = data.m_Script
                raw = script.encode('utf-8', 'surrogateescape') if isinstance(script, str) else bytes(script)
                fn = safe_name(tname, name, obj.path_id)
                open(os.path.join(kind_dir, fn), 'wb').write(raw)
                entry['exported'] = f'{tname}/{fn}'
            elif tname == 'Font':
                fd = getattr(data, 'm_FontData', None)
                if fd:
                    os.makedirs(kind_dir, exist_ok=True)
                    raw = bytes(fd)
                    ext = 'otf' if raw[:4] == b'OTTO' else 'ttf'
                    fn = safe_name(tname, name, obj.path_id) + '.' + ext
                    open(os.path.join(kind_dir, fn), 'wb').write(raw)
                    entry['exported'] = f'{tname}/{fn}'
            elif tname == 'Mesh':
                os.makedirs(kind_dir, exist_ok=True)
                fn = safe_name(tname, name, obj.path_id) + '.obj'
                open(os.path.join(kind_dir, fn), 'w').write(data.export())
                entry['exported'] = f'{tname}/{fn}'
    except Exception as e:
        errors.append({'file': entry['file'], 'path_id': obj.path_id, 'type': tname,
                       'error': f'{type(e).__name__}: {e}',
                       'trace': traceback.format_exc(limit=3)})
    inventory.append(entry)

json.dump(inventory, open(os.path.join(out, 'inventory.json'), 'w'), indent=1)
json.dump(errors, open(os.path.join(out, 'errors.json'), 'w'), indent=1)

counts = {}
for e in inventory:
    counts[e['type']] = counts.get(e['type'], 0) + 1
for k, v in sorted(counts.items(), key=lambda kv: -kv[1]):
    print(f'{v:>6}  {k}')
print(f'objects: {len(inventory)}  errors: {len(errors)}')
print('files:', sorted({e["file"] for e in inventory}))

"""Export scenes, prefabs, materials, animation clips, Resources paths and project settings as JSON.

Usage: venv/bin/python -I export_world.py <unpacked-dir> <assets-dir> <il2cpp-dump-dir>

MonoBehaviour / ScriptableObject fields are decoded with mono_decode.py (class layouts from the
Il2CppDumper output); "fields_complete": true means the decode consumed the object's bytes exactly.

Reads <assets-dir>/manifest.json (from publish_assets.py) to link references to published files.

Object references are written as {"ref": "<serialized file>:<path_id>"} plus "asset": "<published
path>" when the target was published. Null references (path_id 0) become null.
Floats are float32 values written in the shortest decimal form that round-trips to the same float32.
"""
import json
import os
import struct
import sys

import UnityPy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mono_decode import Decoder  # noqa: E402

src, assets_dir, dump_dir = sys.argv[1], sys.argv[2], sys.argv[3]
decoder = Decoder(os.path.join(dump_dir, 'dump.cs'), os.path.join(dump_dir, 'DummyDll'))
data_dir = os.path.join(assets_dir, 'data')

SCENE_NAMES = None
SETTINGS_TYPES = (
    'PhysicsManager', 'TimeManager', 'InputManager', 'TagManager', 'QualitySettings',
    'PlayerSettings', 'AudioManager', 'GraphicsSettings', 'BuildSettings', 'Physics2DSettings',
    'NavMeshProjectSettings', 'MonoManager',
)
# Asset types are referenced, never inlined into scene/prefab JSON.
ASSET_TYPES = {
    'Texture2D', 'Sprite', 'AudioClip', 'Mesh', 'Font', 'Shader', 'Material', 'AnimationClip',
    'AnimatorController', 'Cubemap', 'MonoScript', 'ComputeShader', 'PhysicMaterial', 'LightProbes',
}
SCENE_LEVEL_TYPES = {'RenderSettings', 'LightmapSettings', 'OcclusionCullingSettings', 'NavMeshSettings'}

env = UnityPy.load(src)
manifest = json.load(open(os.path.join(assets_dir, 'manifest.json')))
published = {f'{m["source"]["file"]}:{m["source"]["path_id"]}': m['path'] for m in manifest}

objects = {}
for obj in env.objects:
    objects[f'{obj.assets_file.name}:{obj.path_id}'] = obj


def f32(value):
    for digits in range(6, 10):
        text = f'{value:.{digits}g}'
        candidate = float(text)
        if struct.pack('<f', candidate) == struct.pack('<f', value):
            return candidate
    return value


def resolve_file(assets_file, file_id):
    if file_id == 0:
        return assets_file.name
    name = os.path.basename(assets_file.externals[file_id - 1].path)
    return EXTERNAL_ALIASES.get(name, name)


EXTERNAL_ALIASES = {'unity default resources': 'unity_default_resources'}


def make_ref(assets_file, pptr):
    if pptr['m_PathID'] == 0:
        return None
    key = f'{resolve_file(assets_file, pptr["m_FileID"])}:{pptr["m_PathID"]}'
    ref = {'ref': key}
    if key in published:
        ref['asset'] = published[key]
    return ref


def clean(value, assets_file):
    if isinstance(value, dict):
        if set(value.keys()) == {'m_FileID', 'm_PathID'}:
            return make_ref(assets_file, value)
        return {k: clean(v, assets_file) for k, v in value.items()}
    if isinstance(value, list):
        return [clean(v, assets_file) for v in value]
    if isinstance(value, float):
        return f32(value)
    if isinstance(value, (bytes, bytearray)):
        return value.hex()
    return value


def read_clean(obj):
    return clean(obj.read_typetree(), obj.assets_file)


def is_scalar(value):
    return value is None or isinstance(value, (bool, int, float, str))


def format_json(value, level=0):
    """JSON with 4-space indent; containers holding only scalars stay on one line."""
    pad = '    ' * level
    inner = '    ' * (level + 1)
    if is_scalar(value):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, dict):
        if len(value) == 0:
            return '{}'
        if all(is_scalar(v) for v in value.values()):
            body = ', '.join(f'{json.dumps(k, ensure_ascii=False)}: {format_json(v)}' for k, v in value.items())
            return '{ ' + body + ' }'
        body = ',\n'.join(f'{inner}{json.dumps(k, ensure_ascii=False)}: {format_json(v, level + 1)}'
                          for k, v in value.items())
        return '{\n' + body + '\n' + pad + '}'
    if len(value) == 0:
        return '[]'
    if all(is_scalar(v) for v in value):
        return '[' + ', '.join(format_json(v) for v in value) + ']'
    body = ',\n'.join(inner + format_json(v, level + 1) for v in value)
    return '[\n' + body + '\n' + pad + ']'


def write_json(rel, payload):
    path = os.path.join(data_dir, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write(format_json(payload))
        f.write('\n')


def mono_info(obj):
    raw = obj.get_raw_data()
    sc_file, sc_pid = struct.unpack_from('<iq', raw, 16)
    script_key = f'{resolve_file(obj.assets_file, sc_file)}:{sc_pid}'
    name_len = struct.unpack_from('<I', raw, 28)[0]
    name = raw[32:32 + name_len].decode('utf-8', 'replace')
    fields_offset = (32 + name_len + 3) & ~3
    info = {'enabled': raw[12] == 1, 'script': None, 'script_ref': script_key if sc_pid != 0 else None}
    if name:
        info['name'] = name
    if script_key in objects:
        ms = objects[script_key].read()
        info['script'] = f'{ms.m_Namespace}.{ms.m_ClassName}'.lstrip('.')
        assets_file = obj.assets_file

        def resolve(file_id, path_id):
            return make_ref(assets_file, {'m_FileID': file_id, 'm_PathID': path_id})

        result = decoder.decode(info['script'], raw, fields_offset, resolve)
        info['fields_complete'] = result['complete']
        if result['complete']:
            info['fields'] = clean(result['fields'], assets_file)
        else:
            info['fields_error'] = {k: v for k, v in result.items() if k not in ('complete', 'fields')}
    elif len(raw) > fields_offset:
        info['fields_error'] = 'script not found'
    return info


def export_game_object(key):
    go_obj = objects[key]
    go = go_obj.read_typetree()
    node = {
        'ref': key,
        'name': go['m_Name'],
        'active': go['m_IsActive'],
        'layer': go['m_Layer'],
        'tag': go['m_Tag'],
        'components': [],
        'children': [],
    }
    transform_key = None
    for comp in go['m_Component']:
        pptr = comp['component']
        comp_key = f'{resolve_file(go_obj.assets_file, pptr["m_FileID"])}:{pptr["m_PathID"]}'
        comp_obj = objects.get(comp_key)
        if comp_obj is None:
            node['components'].append({'type': None, 'ref': comp_key, 'missing': True})
            continue
        ctype = comp_obj.type.name
        entry = {'type': ctype, 'ref': comp_key}
        if ctype == 'MonoBehaviour':
            entry.update(mono_info(comp_obj))
        else:
            data = read_clean(comp_obj)
            data.pop('m_GameObject', None)
            if ctype in ('Transform', 'RectTransform'):
                transform_key = comp_key
                data.pop('m_Children', None)
                data.pop('m_Father', None)
            entry['data'] = data
        node['components'].append(entry)
    return node, transform_key


def export_hierarchy(file_name):
    """Return root GameObject trees of one serialized file, in Transform order."""
    nodes = {}
    transform_of = {}
    for key, obj in objects.items():
        if obj.assets_file.name == file_name and obj.type.name == 'GameObject':
            node, tkey = export_game_object(key)
            nodes[key] = node
            if tkey is not None:
                transform_of[tkey] = key
    roots = []
    for tkey, go_key in transform_of.items():
        t = objects[tkey].read_typetree()
        node = nodes[go_key]
        for child in t['m_Children']:
            child_key = f'{resolve_file(objects[tkey].assets_file, child["m_FileID"])}:{child["m_PathID"]}'
            child_go = transform_of.get(child_key)
            if child_go is not None:
                node['children'].append(nodes[child_go])
        if t['m_Father']['m_PathID'] == 0:
            roots.append(node)
    orphans = [n for k, n in nodes.items() if k not in transform_of.values()]
    return roots + orphans, len(nodes)


def flatten(roots):
    """Depth-first flat node list; children/parent become refs (keeps JSON indentation shallow)."""
    flat = []

    def visit(node, parent_ref):
        children = node['children']
        flat.append({**node, 'parent': parent_ref, 'children': [c['ref'] for c in children]})
        for child in children:
            visit(child, node['ref'])

    for root in roots:
        visit(root, None)
    return flat


# --- project settings ---------------------------------------------------------------------------
for obj in env.objects:
    if obj.type.name in SETTINGS_TYPES:
        write_json(f'settings/{obj.type.name}.json', read_clean(obj))
        if obj.type.name == 'BuildSettings':
            SCENE_NAMES = [os.path.splitext(os.path.basename(s))[0] for s in obj.read().scenes]

# --- Resources paths (used by Resources.Load and PhotonNetwork.Instantiate) -----------------------
resources = {}
for obj in env.objects:
    if obj.type.name == 'ResourceManager':
        tree = obj.read_typetree()
        for path, pptr in tree['m_Container']:
            ref = make_ref(obj.assets_file, pptr)
            target = objects.get(ref['ref']) if ref is not None else None
            resources.setdefault(path, []).append({
                **(ref or {}),
                'type': target.type.name if target is not None else None,
            })
write_json('resources.json', resources)

# --- materials ----------------------------------------------------------------------------------
materials = {}
for key, obj in objects.items():
    if obj.type.name == 'Material':
        data = read_clean(obj)
        shader = data.get('m_Shader')
        if shader is not None and shader['ref'] in objects:
            shader['name'] = objects[shader['ref']].read().m_ParsedForm.m_Name
        materials[key] = data
write_json('materials.json', materials)

# --- animation clips / controllers (raw, decoded later) -------------------------------------------
for kind in ('AnimationClip', 'AnimatorController'):
    for key, obj in objects.items():
        if obj.type.name == kind:
            data = read_clean(obj)
            safe = ''.join(c if c.isalnum() or c in '._- ' else '_' for c in data['m_Name'])
            write_json(f'animations/{kind}/{safe}__{key.replace(":", "_")}.json', data)

# --- standalone ScriptableObjects ----------------------------------------------------------------
for key, obj in objects.items():
    if obj.type.name != 'MonoBehaviour':
        continue
    go_file, go_pid = struct.unpack_from('<iq', obj.get_raw_data(), 0)
    if go_pid != 0:
        continue
    info = mono_info(obj)
    safe = ''.join(c if c.isalnum() or c in '._- ' else '_' for c in info.get('name', 'unnamed'))
    write_json(f'scriptable/{info["script"]}/{safe}__{key.replace(":", "_")}.json', {'ref': key, **info})

# --- scenes -------------------------------------------------------------------------------------
for index, scene_name in enumerate(SCENE_NAMES):
    file_name = f'level{index}'
    settings = {}
    for key, obj in objects.items():
        if obj.assets_file.name == file_name and obj.type.name in SCENE_LEVEL_TYPES:
            settings[obj.type.name] = read_clean(obj)
    roots, count = export_hierarchy(file_name)
    write_json(f'scenes/{scene_name}.json', {
        'scene': scene_name,
        'build_index': index,
        'file': file_name,
        'settings': settings,
        'game_objects': count,
        'roots': [r['ref'] for r in roots],
        'nodes': flatten(roots),
    })
    print(f'scene {index:>2} {scene_name:<12} {count:>5} game objects')

# --- prefabs / non-scene GameObjects (one file per root) ----------------------------------------
resource_path_of = {}
for path, targets in resources.items():
    for target in targets:
        if target.get('type') == 'GameObject':
            resource_path_of[target['ref']] = path


def safe_path(text):
    parts = [''.join(c if c.isalnum() or c in '._- ()' else '_' for c in part).strip() or '_'
             for part in text.split('/')]
    return '/'.join(parts)


asset_files = sorted({o.assets_file.name for o in env.objects
                      if o.assets_file.name.endswith('.assets') or o.assets_file.name == 'unity_builtin_extra'})
for file_name in asset_files:
    roots, count = export_hierarchy(file_name)
    for root in roots:
        res_path = resource_path_of.get(root['ref'])
        if res_path is not None:
            rel = f'prefabs/resources/{safe_path(res_path)}.json'
        else:
            rel = f'prefabs/{file_name}/{safe_path(root["name"])}__{root["ref"].split(":")[1]}.json'
        write_json(rel, {'file': file_name, 'resources_path': res_path, 'root': root['ref'],
                         'nodes': flatten([root])})
    if count:
        print(f'prefabs {file_name:<26} {count:>5} game objects in {len(roots)} roots')

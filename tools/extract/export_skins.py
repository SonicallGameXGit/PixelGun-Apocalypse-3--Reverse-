"""Skinning data of every published skinned mesh -> assets/data/skins/<model stem>.json.

Usage: venv/bin/python -I export_skins.py <unpacked-dir> <assets-dir>

The OBJ files in assets/models/ carry no skinning. Their vertex order is Unity's own (one OBJ
vertex per Unity vertex, `f i/i/i`), so this sidecar is indexed by the same vertex index.
Values are in Unity mesh space (not mirrored like the OBJ). Floats are float32 written in the
shortest decimal form that round-trips (as in export_world.py).

Unity 2018.3 stores weights in the vertex streams: channel 12 = BlendWeight (float x4),
channel 13 = BlendIndices (UInt32 x4). Bind poses are 4x4 matrices, written row-major
(e00 e01 e02 e03 e10 ...), i.e. mesh space -> bone space.
"""
import json
import os
import struct
import sys

import UnityPy
from UnityPy.helpers.MeshHelper import MeshHandler

src, assets_dir = sys.argv[1], sys.argv[2]
out_dir = os.path.join(assets_dir, 'data', 'skins')

manifest = json.load(open(os.path.join(assets_dir, 'manifest.json')))
published = {(m['source']['file'], m['source']['path_id']): m['path'] for m in manifest if m['type'] == 'Mesh'}


def f32(value):
    for digits in range(6, 10):
        text = f'{value:.{digits}g}'
        candidate = float(text)
        if struct.pack('<f', candidate) == struct.pack('<f', value):
            return candidate
    return value


def format_json(value, level=0):
    """4-space indent; lists of scalars (one per vertex/matrix) stay on one line."""
    pad = '    ' * level
    inner = '    ' * (level + 1)
    if value is None or isinstance(value, (bool, int, float, str)):
        return json.dumps(value)
    if isinstance(value, dict):
        body = ',\n'.join(f'{inner}{json.dumps(k)}: {format_json(v, level + 1)}' for k, v in value.items())
        return '{\n' + body + '\n' + pad + '}'
    if len(value) == 0:
        return '[]'
    if all(not isinstance(v, (list, dict)) for v in value):
        return '[' + ', '.join(json.dumps(v) for v in value) + ']'
    body = ',\n'.join(inner + format_json(v, level + 1) for v in value)
    return '[\n' + body + '\n' + pad + ']'


if os.path.isdir(out_dir):
    for name in os.listdir(out_dir):
        os.remove(os.path.join(out_dir, name))
os.makedirs(out_dir, exist_ok=True)

env = UnityPy.load(src)
written = 0
for obj in env.objects:
    if obj.type.name != 'Mesh':
        continue
    asset = published.get((obj.assets_file.name, obj.path_id))
    if asset is None:
        continue
    mesh = obj.read()
    if len(mesh.m_BindPose) == 0:
        continue
    handler = MeshHandler(mesh)
    handler.process()
    count = handler.m_VertexCount
    weights = handler.m_BoneWeights
    indices = handler.m_BoneIndices
    if weights is None or indices is None or len(weights) != count or len(indices) != count:
        raise SystemExit(f'{asset}: bone weights missing or wrong length')
    bind_poses = []
    for m in mesh.m_BindPose:
        bind_poses.append([f32(getattr(m, f'e{r}{c}')) for r in range(4) for c in range(4)])
    payload = {
        'mesh': asset,
        'source': f'{obj.assets_file.name}:{obj.path_id}',
        'vertex_count': count,
        'root_bone_name_hash': mesh.m_RootBoneNameHash,
        'bone_name_hashes': list(mesh.m_BoneNameHashes),
        'bind_poses': bind_poses,
        'bone_indices': [[int(i) for i in v] for v in indices],
        'bone_weights': [[f32(w) for w in v] for v in weights],
    }
    stem = os.path.splitext(os.path.basename(asset))[0]
    with open(os.path.join(out_dir, stem + '.json'), 'w') as f:
        f.write(format_json(payload))
        f.write('\n')
    written += 1

print(f'skinned meshes: {written}')

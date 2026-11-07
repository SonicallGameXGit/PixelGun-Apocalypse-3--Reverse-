"""Export game shaders (excluding Unity built-in resources) as ShaderLab text with compiled GLSL ES programs.

Usage: venv/bin/python -I export_shaders.py <unpacked-dir> <assets-dir>
"""
import os
import sys

import UnityPy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from unity_compat import export_shader, shader_name  # noqa: E402

src, assets_dir = sys.argv[1], sys.argv[2]
out = os.path.join(assets_dir, 'shaders')
os.makedirs(out, exist_ok=True)
for name in os.listdir(out):
    if name.endswith('.shader'):
        os.remove(os.path.join(out, name))

env = UnityPy.load(src)
used = {}
count = 0
for obj in env.objects:
    if obj.type.name != 'Shader' or obj.assets_file.name in ('unity_default_resources', 'unity_builtin_extra'):
        continue
    shader = obj.read()
    safe = shader_name(shader).replace('/', '__')
    used[safe] = used.get(safe, 0) + 1
    if used[safe] > 1:
        safe = f'{safe}__{obj.path_id}'
    with open(os.path.join(out, f'{safe}.shader'), 'w') as f:
        f.write(export_shader(shader))
    count += 1
print(f'exported {count} shaders')

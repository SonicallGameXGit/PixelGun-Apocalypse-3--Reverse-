# tools/extract: asset extraction pipeline

Reproduces `assets/` from the original build. Python for now (exploration speed); a Rust port (`pga3-extract`) is planned and must produce identical output.

## Requirements

- `brotli` CLI
- Python 3 with [UnityPy](https://github.com/K0lb3/UnityPy) 1.25.4, in a venv at `research/tools/venv`:
  `uv venv research/tools/venv && uv pip install --python research/tools/venv UnityPy==1.25.4`
- For code structure (not assets): [Il2CppDumper](https://github.com/Perfare/Il2CppDumper) (commit `4741d46`), built with .NET (retargeted to `net10.0`), in `research/tools/Il2CppDumper`.

## Steps (run from the repo root)

```bash
B=research/unity-build
# 1. download (see docs/original-build.md for URLs) into $B, then decompress brotli
mkdir -p $B/raw
brotli -d -f -o $B/raw/pga3-v18.data         $B/pga3-v18.data.unityweb
brotli -d -f -o $B/raw/pga3-v18.wasm         $B/pga3-v18.wasm.code.unityweb
brotli -d -f -o $B/raw/pga3-v18.framework.js $B/pga3-v18.wasm.framework.unityweb

# 2. unpack the UnityWebData1.0 container
python3 -I tools/extract/unpack_webdata.py $B/raw/pga3-v18.data "$PWD/$B/unpacked"

# 3. extract every object (raw, includes MonoBehaviour bytes) into research/extracted
research/tools/venv/bin/python -I tools/extract/extract_assets.py "$PWD/$B/unpacked" "$PWD/research/extracted"

# 4. publish curated game-owned assets + manifest into assets/
python3 -I tools/extract/publish_assets.py research/extracted assets

# 5. IL2CPP dump into research/il2cpp-dump (needed by step 6 for MonoBehaviour layouts)
dotnet research/tools/Il2CppDumper/Il2CppDumper/bin/Release/net10.0/Il2CppDumper.dll \
    $B/raw/pga3-v18.wasm $B/unpacked/Il2CppData/Metadata/global-metadata.dat research/il2cpp-dump

# 6. scenes, prefabs, materials, texture sampler settings, decoded script data, settings, animations -> assets/data/
research/tools/venv/bin/python -I tools/extract/export_world.py "$PWD/$B/unpacked" "$PWD/assets" "$PWD/research/il2cpp-dump"

# 7. shaders -> assets/shaders/
research/tools/venv/bin/python -I tools/extract/export_shaders.py "$PWD/$B/unpacked" "$PWD/assets"
```

## Scripts

| Script | Purpose |
|--------|---------|
| `unpack_webdata.py` | `UnityWebData1.0` container → files |
| `extract_assets.py` | every object → raw/decoded files + `inventory.json` (research/extracted) |
| `publish_assets.py` | curated game-owned files → `assets/` + `assets/manifest.json` |
| `export_world.py` | `assets/data/` JSON (see `docs/assets.md` for the format) |
| `mono_decode.py` | decodes stripped MonoBehaviour data with `dump.cs` layouts (Unity 2018.3 rules) |
| `dotnet_meta.py` | reads field flags (`NotSerialized`) from Il2CppDumper's DummyDll; `dump.cs` omits them |
| `export_shaders.py` | game shaders → ShaderLab text with GLSL ES programs |

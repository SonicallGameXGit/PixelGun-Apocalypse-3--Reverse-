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

# 8. skinning (bind poses, bone weights) of the published skinned meshes -> assets/data/skins/
research/tools/venv/bin/python -I tools/extract/export_skins.py "$PWD/$B/unpacked" "$PWD/assets"
```

## The CrazyGames build (Unity 5.3.5, asm.js)

```bash
C=research/crazygames-build
# download pixelgunapocalypse3.2.{datagz,jsgz,memgz} and UnityLoader.js (URLs: docs/crazygames-build.md) into $C, then:
mkdir -p $C/raw
for f in data js mem; do gzip -dc $C/pixelgunapocalypse3.2.${f}gz > $C/raw/pixelgunapocalypse3.2.$f; done
python3 -I tools/extract/unpack_emscripten_package.py $C/UnityLoader.js $C/raw/pixelgunapocalypse3.2.data $C/unpacked
# IL2CPP dump: Il2CppDumper reads the static memory as a wasm data section; it finds only the
# CodeRegistration itself, so both addresses are entered (manual mode, from docs/crazygames-build.md)
python3 -I tools/re/asmjs_mem_to_wasm.py $C/raw/pixelgunapocalypse3.2.mem $C/raw/memory.wasm
printf '12eadc\ne40fc\n' | dotnet research/tools/Il2CppDumper/Il2CppDumper/bin/Release/net10.0/Il2CppDumper.dll \
    $C/raw/memory.wasm $C/unpacked/Il2CppData/Metadata/global-metadata.dat research/crazygames-il2cpp-dump
```

Assets (this is what `assets/` holds since 2026-11-07). The same scripts as steps 3, 4 and 6–8; they detect Unity 5.3 from the serialized files (`unity_compat.py`):

```bash
C=research/crazygames-build
research/tools/venv/bin/python -I tools/extract/extract_assets.py "$PWD/$C/unpacked" "$PWD/research/crazygames-extracted"
python3 -I tools/extract/publish_assets.py research/crazygames-extracted assets
rm -rf assets/data
research/tools/venv/bin/python -I tools/extract/export_world.py "$PWD/$C/unpacked" "$PWD/assets" "$PWD/research/crazygames-il2cpp-dump"
research/tools/venv/bin/python -I tools/extract/export_shaders.py "$PWD/$C/unpacked" "$PWD/assets"
research/tools/venv/bin/python -I tools/extract/export_skins.py "$PWD/$C/unpacked" "$PWD/assets"
```

To compare two published trees (e.g. a Build-7 tree from git history against this one): `research/tools/venv/bin/python -I tools/extract/compare_assets.py <assets-a> <assets-b> <report.json>` (results: `docs/crazygames-build.md`, "Assets").

## Scripts

| Script | Purpose |
|--------|---------|
| `unpack_webdata.py` | `UnityWebData1.0` container → files |
| `unpack_emscripten_package.py` | Unity 5.3 `.data` (Emscripten file package, table in `UnityLoader.js`) → files |
| `extract_assets.py` | every object → raw/decoded files + `inventory.json` (research/extracted) |
| `publish_assets.py` | curated game-owned files → `assets/` + `assets/manifest.json` |
| `export_world.py` | `assets/data/` JSON (see `docs/assets.md` for the format) |
| `mono_decode.py` | decodes stripped MonoBehaviour data with `dump.cs` layouts (Unity 2018.3 or 5.3 rules: 5.3 keyframes have no weights) |
| `unity_compat.py` | Unity 5.3 vs 2018.3 helpers: build version, shader name from `m_Script`, 5.3 sub-program blobs read little-endian |
| `compare_assets.py` | compares two published asset trees by content (pixels / bytes) and reports same, changed and one-sided files |
| `dotnet_meta.py` | reads field flags (`NotSerialized`) from Il2CppDumper's DummyDll; `dump.cs` omits them |
| `export_shaders.py` | game shaders → ShaderLab text with GLSL ES programs |
| `export_skins.py` | bind poses + per-vertex bone indices/weights of skinned meshes → `assets/data/skins/` |

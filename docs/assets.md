# Assets

Pipeline: `tools/extract/` (see its README). Output: `assets/` + `assets/manifest.json` (each file → original Unity serialized file + `path_id`, name, size/audio info).

## Published (2026-11-06)

| Type | Count | Folder | Format |
|------|-------|--------|--------|
| Texture2D | 277 | `assets/textures/` | PNG (decoded) |
| Sprite | 41 | `assets/ui/sprites/` | PNG |
| AudioClip | 48 | `assets/audio/` | `.m4a`, the original AAC stream, byte-exact, not re-encoded |
| Mesh | 173 | `assets/models/` | OBJ (static geometry only, no skinning yet) |
| Font | 11 | `assets/fonts/` | TTF |

Excluded: Unity built-in resources (`unity_default_resources`, `unity_builtin_extra`); 12 empty 0×0 `Font Texture` objects (runtime dynamic font atlases, nothing to extract).

Names with a `__<path_id>` suffix are same-named distinct objects.

## Audio

- Every clip is `m_CompressionFormat = 7` (AAC), stored as an MP4 container in `sharedassets0.resource` / `resources.resource` and referenced by `m_Resource` (source, offset, size).
- Music: only **`muzica lobby.m4a`** (mono, 44.1 kHz, 22.76 s). No other music track exists in this build (**observed**: all 48 clips listed). If in-game music existed, it was streamed from elsewhere or belongs to a different version (**unknown**).

## Fonts: licensing note

Some fonts bundled by the original developers are commercial third-party fonts: `ebrima.ttf` / `ebrimabd.ttf` (Microsoft Ebrima) and `_-_IMPACT.ttf` (Impact, Monotype). Redistribution may not be permitted; decide before publishing the repo.

## Not yet extracted

- Materials (299), shaders (55, compiled for WebGL GLSL ES), animation clips (85), animator controllers (3).
- Skinned meshes / skeletons (characters, weapons) → glTF with bones and animations.
- Scene data (20 scenes): GameObject hierarchy, transforms, colliders, lightmaps → our own scene format.
- MonoBehaviour field data (1829 objects). Raw bytes are in `research/extracted/MonoBehaviour/<Script>/`; typetrees are stripped, so fields must be decoded with the class layouts from `research/il2cpp-dump/dump.cs` (proven to work: `PhotonServerSettings` decodes byte-exact, see `docs/protocol.md`).

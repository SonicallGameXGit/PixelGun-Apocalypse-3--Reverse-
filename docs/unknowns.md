# Unknowns

| # | Question | Needed to resolve |
|---|----------|-------------------|
| 1 | Scene data, materials, shaders, animations, skinned meshes not yet extracted | Extend `tools/extract` (see `docs/assets.md`) |
| 2 | Exact Photon connect flow, AppVersion string, room/player properties, serialization | wasm decompilation + runtime capture (PUN 1.91 on Photon Cloud is established) |
| 3 | Whether GameArter overrides the Photon AppID at runtime / is required to play | Runtime capture |
| 4 | Method bodies (gameplay math: movement, damage, recoil, spread) | Decompile `pga3-v18.wasm` with function names from `script.json` |
| 5 | Is there music beyond the 22.8 s lobby loop (the "OST")? | Play the original and compare; check network for streamed audio |
| 6 | Redistribution of bundled commercial fonts (Ebrima, Impact) | User decision |

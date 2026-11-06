# Unknowns

| # | Question | Needed to resolve |
|---|----------|-------------------|
| 1 | Animations (Mecanim muscle clips) and skinned meshes not yet converted; legacy `Animation` components reference null clips | Muscle-clip decoder + glTF exporter (see `docs/assets.md`) |
| 2 | Exact Photon connect flow, AppVersion string, room/player properties, serialization | wasm decompilation + runtime capture (PUN 1.91 on Photon Cloud is established) |
| 3 | Whether GameArter overrides the Photon AppID at runtime / is required to play | Runtime capture |
| 4 | Method bodies (gameplay math: movement, damage, recoil, spread) | Decompile `pga3-v18.wasm` with function names from `script.json` |
| 5 | Is there music beyond the 22.8 s lobby loop (the "OST")? | Play the original and compare; check network for streamed audio |
| 6 | Redistribution of bundled commercial fonts (Ebrima, Impact) | User decision |

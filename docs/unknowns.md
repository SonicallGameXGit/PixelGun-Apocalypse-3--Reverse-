# Unknowns

| # | Question | Needed to resolve |
|---|----------|-------------------|
| 1 | Animations (Mecanim muscle clips) and skinned meshes not yet converted; legacy `Animation` components reference null clips | Muscle-clip decoder + glTF exporter (see `docs/assets.md`) |
| 2 | Exact Photon connect flow, AppVersion string, room/player properties, serialization | Decompile the PUN/menu code with `tools/re/` + runtime capture (PUN 1.91 on Photon Cloud is established) |
| 3 | Whether GameArter overrides the Photon AppID at runtime / is required to play | Runtime capture |
| 4 | Remaining method bodies (damage, recoil, spread, ladders, `PlayerDamage`). Decompilation now works (`tools/re/`); movement is done (`docs/gameplay/movement.md`) | Decompile per system as it is implemented |
| 5 | Is there music beyond the 22.8 s lobby loop (the "OST")? | Play the original and compare; check network for streamed audio |
| 6 | Redistribution of bundled commercial fonts (Ebrima, Impact) | User decision |
| 7 | PhysX CharacterController details the client infers (step validation, no step while moving up, horizontal side-pass slide normal, overlap recovery, side-pass iteration count) | Measure in the original: walk into ledges of 0.25/0.35 m, jump at a ledge edge, run along slanted walls; compare positions |
| 8 | Mouse look scale: Unity WebGL `Mouse X/Y` units (raw pointer-lock pixels × 0.1 is assumed) and whether `CF2Input` axes fall back to Unity `Input` on desktop | Turn 1000 px in the original at sensitivity 1.8 and measure the angle |

Resolved:

- Renderer fidelity (former row 7): the user compared screenshots of the original and this client at the same spot on Bridge and found them identical; no visible issues on Castle.

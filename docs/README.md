# docs/

Distilled, evidence-backed knowledge about the original game. Keep each claim labelled as **observed**, **inferred** or **unknown**, with a pointer to the evidence (REA evidence ID, capture file, or asset path).

Existing:

- `original-build.md`: Unity WebGL build layers (brotli → UnityWebData → UnityFS) and IL2CPP metadata.
- `code-structure.md`: assemblies, game classes, scenes.
- `protocol.md`: multiplayer (PUN Classic 1.91 / Photon Cloud), server settings, RPC list.
- `assets.md`: extraction pipeline, published asset counts, gaps.
- `architecture.md`: client crates and rationale, modules, coordinate conventions, renderer (shader ports), how to run.
- `unknowns.md`: open questions.
- `gameplay/movement.md`: player movement, input, mouse look, camera bob, CharacterController model (decompiled `FPScontroller` & co.).

Planned:

- `gameplay/`: weapons, damage, ladders, modes, progression, balance tables.

Tooling for reading the original code (wasm2c + IL2CPP names): `tools/re/README.md`.

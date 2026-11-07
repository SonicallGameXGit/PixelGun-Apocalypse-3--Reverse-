# research/

Raw reverse-engineering material. **Everything here except this file is gitignored.**

- `source/page/`: saved game page (`PixelGun Apocalypse 3 (Source).html` + `_files/`), originally from <https://www.pacogames.com/game/pixel-gun-apocalypse-3/>. Build manifest: `/game/pixel-gun-apocalypse-3/Build-7/pga3-v18.json`.
- `unity-build/`: original Unity WebGL build files (manifest, `.data.unityweb`, wasm/asm code, framework). Download here; never commit.
- `source/crazygames-page/`: saved CrazyGames page of the live build (`docs/crazygames-build.md`).
- `crazygames-build/`: its build files (`*gz`, `UnityLoader.js`), `raw/` (gunzipped + `memory.wasm`), `unpacked/` (file package contents). Never commit.
- `crazygames-il2cpp-dump/`: Il2CppDumper output for it, plus `asmjs_index.json`.
- `captures/`: HAR / pcap / websocket captures and screenshots of the original game. May contain tokens: sanitize before distilling into `docs/`.
- `notes/`: scratch notes and REA evidence bundles.

Distilled, reproducible conclusions belong in `../docs/`, extracted assets in `../assets/`.

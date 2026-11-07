# tools/net

Recording and decoding the original game's Photon traffic. The findings are in `docs/protocol.md`.

## Recording a session

The CrazyGames build (Unity 5.3.5) runs on `games.crazygames.com/en_US/block-pixel-gun-apocalypse-3/index.html`. That page waits for its parent frame to post the loader configuration, so it can be opened on its own and the hook installed before any socket exists:

1. Open that page in its own tab and open DevTools.
2. Paste `capture-hook.js` into the console.
3. Start the game:

   ```js
   const base = 'https://files.crazygames.com/webgl/blocky-pixel-apocalypse-3/Release/'
   window.postMessage({ type: 'unity54config', loaderOptions: { module: { memUrl: base + 'pixelgunapocalypse3.2.memgz', codeUrl: base + 'pixelgunapocalypse3.2.jsgz', dataUrl: base + 'pixelgunapocalypse3.2.datagz' }, unityLoaderUrl: base + 'UnityLoader.js' } }, '*')
   ```

4. Play what you want to record.
5. Run `__wsSave()` and move the downloaded `ws-capture.json` to `research/captures/` (gitignored).

The capture contains the Photon auth secret (parameter 221) and the user id (225). Never commit it.

## Decoding

```bash
python3 -I tools/net/photon_dump.py research/captures/crazygames/ws-2026-11-07.json --skip-pings --redact > dump.txt
```

This prints one line per Photon message: the direction, the server, the message type, the op/event code and the decoded Protocol16 parameters. PUN's custom types are shown as `V3(...)`, `Q(...)` and `P(actor)`. `--redact` replaces parameters 221 and 225.

## Test fixture

```bash
python3 -I tools/net/make_fixture.py research/captures/crazygames/ws-2026-11-07.json > docs/fixtures/photon-crazygames-frames.txt
```

This keeps up to 4 frames of each (direction, type, code) and drops every frame that carries 221 or 225. The client's codec tests (`net/photon.rs`, `net/pun.rs`) decode and re-encode each line and check the result is byte-exact.

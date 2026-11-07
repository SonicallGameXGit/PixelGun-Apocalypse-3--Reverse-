// Records every WebSocket frame of the page (the Unity build's Photon traffic) for photon_dump.py.
// Paste into the DevTools console of the game's own page BEFORE the Unity loader opens a socket
// (see tools/net/README.md), play, then run `__wsSave()` to download the capture as JSON:
//     [{ t: ms, dir: 'open' | 'out' | 'in' | 'close', url | ws: string, d: base64, code }, ...]
// The capture contains the Photon auth secret and user id: keep it in research/ (gitignored).
(function () {
    const w = window
    if (w.__wsHooked === true) {
        return
    }
    w.__wsHooked = true
    w.__wsLog = []
    const Orig = w.WebSocket
    const b64 = (buf) => {
        const u = new Uint8Array(buf)
        let s = ''
        for (let i = 0; i < u.length; i++) {
            s += String.fromCharCode(u[i])
        }
        return btoa(s)
    }
    function Hooked(url, protocols) {
        const ws = protocols === undefined ? new Orig(url) : new Orig(url, protocols)
        w.__wsLog.push({ t: performance.now(), dir: 'open', url: String(url), protocols: protocols === undefined ? null : String(protocols) })
        const origSend = ws.send.bind(ws)
        ws.send = function (data) {
            let d
            if (data instanceof ArrayBuffer) {
                d = b64(data)
            } else if (ArrayBuffer.isView(data)) {
                d = b64(data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength))
            } else {
                d = 'TXT:' + String(data)
            }
            w.__wsLog.push({ t: performance.now(), dir: 'out', ws: String(url), d: d })
            return origSend(data)
        }
        ws.addEventListener('message', (e) => {
            const p = e.data
            const t = performance.now()
            if (p instanceof ArrayBuffer) {
                w.__wsLog.push({ t: t, dir: 'in', ws: String(url), d: b64(p) })
            } else if (p instanceof Blob) {
                p.arrayBuffer().then((b) => w.__wsLog.push({ t: t, dir: 'in', ws: String(url), d: b64(b) }))
            } else {
                w.__wsLog.push({ t: t, dir: 'in', ws: String(url), d: 'TXT:' + String(p) })
            }
        })
        ws.addEventListener('close', (e) => {
            w.__wsLog.push({ t: performance.now(), dir: 'close', ws: String(url), code: e.code })
        })
        return ws
    }
    Hooked.prototype = Orig.prototype
    Hooked.CONNECTING = 0
    Hooked.OPEN = 1
    Hooked.CLOSING = 2
    Hooked.CLOSED = 3
    w.WebSocket = Hooked
    w.__wsSave = function () {
        const a = document.createElement('a')
        a.href = URL.createObjectURL(new Blob([JSON.stringify(w.__wsLog)], { type: 'application/json' }))
        a.download = 'ws-capture.json'
        a.click()
        return w.__wsLog.length + ' frames'
    }
})()

# Menus: boot, server choice, lobby, room creation, loading, in-room menu

Spec of the CrazyGames build's (Unity 5.3.5, IL2CPP to asm.js) menu flow and of what the client does for it.
Every statement is labelled **observed** (decompiled code or extracted data), **inferred** or **unknown**.

Evidence: decompiled with `tools/re/asmjs_func.py` into `research/notes/crazygames/` (`connectmenu.js`,
`serverchose.js`, `roommultiplayermenu.js`; gitignored); scene data `assets/data/scenes/Main.json`;
the lobby skin `assets/data/scriptable/UnityEngine.GUISkin/1 lobby swat__sharedassets0.assets_141.json`;
button clips `assets/data/animations/AnimationClip/*sharedassets0.assets_11x/12x*`.
Code: `menu.rs` (flow), `ui.rs` (uGUI subset), `layout.rs` (`GUILayout`), `gui.rs` (skins, fonts), `main.rs` (screens).

## Flow

```
boot picture -> Main: "Choose Server" -> (connected) lobby: room list <-> create room
   -> JoinRoom / CreateRoom -> OnJoinedRoom -> fade + 1 s -> map scene (the game)
   -> Tab: "Leave Room" -> OnLeftRoom -> Main (already connected: straight to the lobby)
```

- **Observed:** `Main` (level 0) is the first scene. The build has no loading screen of its own: in the CrazyGames embed
  `UnityProgress` is a stub and the host page draws the loader.
  `ConnectMenu.FadeScreen` draws `textures/loading crazy games.png` (2048x1024, logo and "Loading...") over the whole
  window during scene changes.
- **Client:** the boot screen shows that picture for 0.6 s (the host page's loader has no equivalent here).
- **Observed:** `ServerChose.Update`: while `PhotonNetwork.connected`, the server window is deactivated and the
  `__Lobby1` object (the `ConnectMenu` OnGUI lobby with its own camera) activated.
  `ServerChose.OnDisconnectedFromPhoton` sets `ConnectMenu.chooseserve = false`, deactivates the lobby and shows the window
  again. `chooseserverondisconect` calls `PhotonNetwork.Disconnect`.
- **Client:** a dropped connection (`ClientEvent::Failed`) returns to the server window; "Change Server" does the same.

## `Main` scene: the server window (uGUI)

All **observed** from `Main.json`.

Canvas: `ui Lobby` (position -7.786, -40.466, 13.234) > `Canvas` (Screen Space - Camera, GUI camera `level0:83`, plane
distance 100, `CanvasScaler` ScaleWithScreenSize 800x600, match = 1: the height) > `MainMenu` (nested canvas, centre
anchored, pos (0,-46), size 1000x600) > `Window`.

| Object | Value |
|---|---|
| Main Camera | world (0,0,-100), fov 40, near 0.1, far 1000, solid clear (0.0616, 0.0949, 0.147) |
| GUI Camera | world (0,12.534,-100), fov 40, near 0.3, far 1000, depth only, culling mask 32 |
| Canvas scale | 2·100·tan(20°)/600 = 0.1213 world units per canvas unit; canvas is 600·aspect x 600 units |
| Background | node `webgl` (`level0:58`): built-in Cube at (-2.8,2,83.3), scale (-277.59,-141.66,7.07), `Mobile/Diffuse` with `textures/1.png` (1024x512) |
| Light / ambient | directional, intensity 1, rotation quat (0.408,-0.2346,0.1094,0.8754); ambient flat white |
| Music | `ui Lobby` has an `AudioSource` playing `audio/muzica lobby.m4a`, volume 0.2, play on awake (**not played by the client yet**) |

`Window` (`level0:29`): anchors (0.8,0.5)-(1,0.5), pivot 0.5, anchored (-418,15), sizeDelta (300,587.9) so the rect is
500x587.9 at local (-18,15,-147); scale 0.678; rotated 1.96° about Y; its `Image` has alpha 0.
Its `TiltWindow` (range (-7,-3)) is serialized **disabled**, so the window does not follow the mouse
(`ui.rs` ignores disabled scripts; the code is implemented for when a build enables it:
`rotation = start * Euler(-mRot.y*range.y, mRot.x*range.x, 0)`, `mRot` lerped to the mouse at `dt*5`).

Children in draw order (positions are anchored positions in the Window; fonts under `assets/fonts/`):

| Child | Detail |
|---|---|
| `title paco` | Text "Block Pixel Gun Apocalypse 3", size 51, bold, red, `Apocalypse.ttf`, centred; pos (-63,402), size 599x73, scale 1.474 |
| `Choose Server` | Image `ui/sprites/small-red-button-hi.png`, pos (0,277), 324x85; label "Choose Server" (size 44, white, `cool Example Font.ttf`) |
| `Europe` | Button, pos (0,200), 250x70; OnClick `ServerChose.eu` |
| `America` | Button, pos (-2,123), 250x70; `us` |
| `Asia` | Button, pos (0,41), 250x70; `asia` |
| `More Games` | Button, pos (2,-40), 250x70; `Link.OpenCrazy` |
| `facebook` / `playstore` / `youtube` | 92x70 buttons at (131,-132) / (0,-132) / (-129,-132): `OpenFacebook`, `OpenIoanMita2016`, `OpenYoutube`; sprites `like.png`, `playstore.png`, `youtube.png` |

Every button has a `Background` child (stretch, sprite `ui/sprites/menu BlueSquareButton 1.png`, colour (1,1,1,0.3255))
holding a `Label` (size 45, black, `cool Example Font.ttf`; Europe's offset (0,6.06)).
Button transition = Animation (`SF Button` controller, 0.1 s): `Highlighted` and `Pressed` set the Background colour to
opaque white, `Normal`/`Disabled` restore the default. The clips' other bindings (attribute 2033536083) target a missing
component and do nothing. **Client:** the hover animation lerps the Background alpha 0.3255 -> 1 over 0.1 s.

`ServerChose` methods call `PhotonNetwork.ConnectToRegion(code, GameVersion)`: `eu` 0, `us` 1, `asia` 2, `jp` 3, `au` 5.
Only `eu`, `us` and `asia` have buttons. **Client:** `Client::connect(profile, region, name)` with `eu`/`us`/`asia`.

Link URLs (**observed**; opened with `xdg-open`): `https://www.crazygames.com/t/mentolatux` (More Games),
`https://www.facebook.com/MultiplayerAndroidGames/`, `https://play.google.com/store/apps/dev?id=5903514895660400064`,
`https://www.youtube.com/channel/UCNxM6ECVEYNLXJepiyWqA6A`; `OpenPaco` is `http://www.pacogames.com/links/pixel-gun-apocalypse-3.php?more`.

Inactive in `Main`: `Titlee swat`, the "Please Read" panel (Settings window, More Games sub-buttons, Back), `UnityAds`,
a second `EventSystem`, and a "Particle System" at (-100,-12.7,-107.4) (looping, lifetime 40; **not rendered**).
The `Credits` scene is only loaded by `X Redirect Link.OnMouseUpCredits`, which nothing reachable from `Main` calls.

**Background orientation (inferred, unknown):** the cube's scale is (-x,-y) but I could not read its UVs; the client draws
`1.png` upright (designer intent) as the cube's front face with a brightness factor 1.557 (ambient 1 plus the light's
`N.L` of 0.557, clamped). `docs/unknowns.md` #37.

**Client:** `ui.rs` loads the `Canvas` subtree of `Main.json`, lays out RectTransforms from anchors, and draws each
Image/Text as a quad through the GUI camera's view-projection (perspective-correct); text is rasterised at the node's
on-screen pixel size. Hit testing unprojects the mouse ray onto each button's plane.

## `ConnectMenu`: the lobby (IMGUI)

Skin "lobby swat" (`sharedassets0.assets:141`), `GUI.depth = -2`, `GUI.enabled = connected` (disabled draws at half
alpha: **inferred**). Camera `__Lobby1/Camera`: position (0,113,-247), rotation x 20°, fov 21, near 0.3, far 600, solid
black; it renders the same background cube from a pitched view (so the picture is lower and a black band shows under it:
that follows from the data).

Skin styles (**observed**, JSON keys are lowercase):

| Style | Font, size | Notes |
|---|---|---|
| box | `A A A A A ebrimabd.ttf` 20, centre, wordWrap | border 4, margin (1,0,0,0), padding (-12,14,-2,0), bg `ActionBar_SpellSlot_PressOverlay.png`, text (0,1,0.09) |
| button | `Apocalypse.ttf` 27, centre | bg `gray-square-button-50203.png` (black text); hover `BlueSquareButton.png` (text (0.434,0,0)); active `MobaActionBar_SpellSlot_Background.png` (white) |
| label | `lOOBYmENU.ttf` 16, upper left, rich text | margin (0,34,0,0): a 34 px right margin; bg `ActionBar_SpellSlot_CooldownOverlay.png`; text (0,1,0.5) |
| textField | `A A A A A ebrimabd.ttf` 20, centre | border 4, padding (4,4,4,-16), contentOffset (-1.68,-8.33); focused bg `SpellSlot_Type0_HoverOverlay.png`; text (0,1,0.01) |
| window | `A A A A A ebrimabd.ttf` 17, upper centre | padding (0,15,-5,0), overflow (26,34,56,21), bg black alpha 128 |
| verticalScrollbar | | fixedWidth 55, thumb `2 SF Title 1.png` (border 0,0,6,6, padding top 45) |

`ConnectMenu.Start` (**observed**): increments PlayerPrefs `GamePlays`; `newRoomName = "Room " + rand(111..9999)`;
`playerName = "Player " + rand(111..9999)`, replaced by PlayerPrefs `PlayerName` when set; reads `Kills`, `Deaths`, `Clan`;
`maxPlayersOptions` 2,4,...,16 (default index 7); `timeOptions` 5,10,15,20 (default index 3, sent as minutes·60);
`gameMode = "TDM"`; `selectedMap = 0`. `Update`: not connected -> `ConnectUsingSettings("V2.0")` every 3 s (**the client's
own Photon connection replaces this**); `MyLevel` from the `Level` list's kills thresholds
(0,10,30,40,50,60,70,80,90,100,150,200,...,2600: 50 levels) against PlayerPrefs `Kills`.
**Client:** PlayerPrefs are a JSON file `$XDG_DATA_HOME/pga3-client/prefs.json` (`PlayerName`, `Clan`, `Kills`,
`Deaths`, `GamePlays`); the kills and deaths are not yet updated by the game (`docs/unknowns.md` #18).

`OnGUI`: draws `Top.png` (fully transparent, a no-op), then `BeginArea(Rect(W/2-370, H/2-237, 750, 475),
"Rooms: N                              Players: N", window style)` around `ShowConnectMenu`; the style's overflow
extends the background past the area; when not connected a Box "Connecting..." (150x30, centred) is drawn; then `FadeScreen`.
Rooms/Players count the listed rooms and their players.

### Room list (`createRoom == false`)

`Space(10)`, then (**observed** order, sizes in px):

1. `Label "Player Name: "` + `TextField(max 15, Width 390, Height 22)`.
2. `Label "Search room name: "` + `TextField(max 17, Width 360, Height 22)`.
3. `Label "Password: "` (Width 80) + `TextField(max 17, Height 22)`; both turn red while `redtimer > 0` (set to 100 by the
   lock button, `-= dt*80`).
4. `BeginScrollView(Width 660, Height 225)` with one `BeginHorizontal("box")` row per listed room, then `EndScrollView`.
   A room is listed when its name contains `PixelWarfare001v` and matches the search text. Rooms whose name contains
   `--redirecttopaco...` or `PixelWarfare002v` redirect to external URLs and are not rows. A row:
   `Label(name without all [WEP01..09] tags and without its last 16 chars, Width 170)`, `Label(MapName, Width 90)`,
   `Label("players/max", Width 40)`, `FlexibleSpace`, the lock `Button` (texture `Lock.png`, 30x30) when the room has a
   `Password` property different from the typed one, and `Button("Join Room", Width 180)`.
   Join = `PhotonNetwork.JoinRoom(name)` after `CheckPlayerNameAndRoom` and saving PlayerName and Clan.
5. `Space(5)`; the level row (shown because `checkwebforpirate.SuperWeapons` is 1): `Box("Level: N", Height 32)`,
   `Box(RankTextures[level-1], 62x62)`, `Box("Kills: n")`, `Box("Deaths: n")`, `Box("Game plays: n")` (Height 32). A
   pacogames "full game" button before it is skipped for the same reason.
6. `FlexibleSpace`, then a row with `FlexibleSpace`, `Button("Change Server", 230x55)` and `Button("Create Room", 230x55)`
   (sets `createRoom = true` and saves PlayerPrefs).

`CheckPlayerNameAndRoom`: spaces are removed from player name, clan and room name; an empty name becomes
`"Player" + rand(111..999)`, an empty room name `"Room Name " + rand(111..999)`.
Search matching is **inferred** as a case-insensitive substring test.

### Room creation (`createRoom == true`)

Rows: `Room Name: ` (Width 130) + field; `Password: ` + field; `Round Time: ` + buttons 5/10/15/20 (55x25);
`Max Players: ` + buttons 2..16 (55x25); `Game Mode: ` + `TeamDeathmatch` (300x30, "TDM") and `Deathmatch` (200x30, "DM"),
`Space` between rows. Selected options are tinted `GUI.color = green`. Then a horizontal group:
`BeginScrollView(320x200)` with one Button (Height 50) per map (selected green), `Space`, a vertical column of weapon
buttons `Pistol` [WEP02], `M242` [WEP03], `Shotgun` [WEP04], `Sniper` [WEP05], `Launcher` [WEP06] (150x28; green when the
tag is in `WEAPONSALLOW`, red when not; click toggles) and the map preview `Label` (230x160). `FlexibleSpace`, then
`Button("Main Menu", 370x40)`, `FlexibleSpace`, `Button("Continue", 370x40)`.

Maps (`allMaps`, **observed**): Bridge, Crash, Pete, Snow, Village, Fight, Scaz, Castle, Tunnels and the nine `ZM_*`
variants; previews `textures/<name>.png`. The "available only on www.pacogames.com" yellow button is shown for the names
"Spider house", "Water bridges" and "Split base" only; none is in this list, so it never appears.
(The weapon toggles' tag order after re-adding is **inferred**: the client keeps the canonical order.)

`Continue` (`CheckPlayerNameAndRoom`, `PhotonNetwork.player.name = name`) calls
`CreateRoom(newRoomName + ": " + gameMode + WEAPONSALLOW + "PixelWarfare001v", true, true, maxPlayers, props, lobbyProps)`
with props `MapName`, `RoundDuration` (minutes·60), `GameMode`, `Password` and the same four keys as lobby properties:
exactly what `online::new_room` already sent (`docs/protocol.md`).

### Loading (`OnJoinedRoom`, `LoadMap`, `FadeScreen`)

`OnJoinedRoom` sets `connectingToRoom` and calls `LoadMap(room.MapName)`: `isMessageQueueRunning = false`, `fadeDir = 1`,
wait 1.0 s, `backgroundLoadingPriority = 4`, `LoadLevelAsync(scene)`. `FadeScreen` raises `fadeValue` by `dt*15` (to 1) and
draws `loading crazy games.png` over the window with that alpha. `OnPhotonJoinRoomFailed`, `OnPhotonCreateRoomFailed`
(only a log line plus clearing `connectingToRoom`), `OnConnectedToPhoton` (leaves a current room), `OnLeftRoom`.
**Client:** the same fade and 1 s delay, then `create()` loads the map (the load is synchronous, so the window shows the
last fade frame meanwhile). Failures are logged and the lobby stays. The connected `Client` is handed to the game with
`Online::from_events`.

### IMGUI layout (`layout.rs`)

Unity's `GUILayout` is reproduced from its group algorithm (recalled from the engine, **inferred**; the lobby's look follows
from it, no original screenshot exists to compare): leaf min/max from `GUIStyle.CalcSize` or the `Width`/`Height` options;
margins of neighbours collapse to the larger one; a styled group (or one with a user size) turns child margins into
padding; along an axis children share the free space by a common factor between their min and max, and once all are at
max the stretchable ones (`stretchWidth` of the style; `FlexibleSpace` along the parent's axis only) share the rest;
a group with a user size has exactly that size and a scroll view's content keeps its natural height (scrolling).
For example the label (stretch) and the fixed 390 px field of "Player Name:" put the field at the right end of the row.

## In-room menu (`RoomMultiplayerMenu`)

**Observed:** Tab opens the menu with `Resume` and `Leave Room`; `LeaveRoom` calls `PhotonNetwork.LeaveRoom` when connected;
`OnLeftRoom` and `OnDisconnectedFromPhoton` both start `LoadMap("Main")`. Strings: " [Tab - MENU]", " [P -FullScreen on/off]",
"Room: ", "Mouse Sensitivity:" with "<" and ">", "Player List", "Controls", "All Players", "- Spectators -", "Join",
"Kills: ", "Deaths: ", "Ping: ". Controls list: "Tab - Pause Menu", "P - Fullscreen", "T - Chat / Enter - send", "C - crouch",
"Left Ctrl - prone", "LMB - fire", "RMB - aim", "R - reload", "Left Shift - run", "Space - jump",
"Buttons 1,2,3,4,5,6,7,8 - weapon change".

**Client (reduced):** when the game was started from the menu, Tab opens a window with "Room: <name>", `Resume`, `Leave Room`
and the controls list (lobby skin); the game keeps running without input. `Leave Room` drops the game and the connection
and shows the lobby again while a new connection is made (the original stays on the master server). The player list, the
scoreboard, sensitivity and the rest of the menu are `docs/parity-checklist.md` rank 1 and not done. GL objects of the left game
are not freed (`SceneRenderer` has no `Drop`): each round trip leaks the previous map's GPU memory.

## Running and verifying

`pga3-client` (no scene, no test option) starts at the boot picture and runs the flow. Naming a scene, `--dummy`,
`--online`, `--camera`, `--spawn`, `--fly`, `--weapon`, `--fire` or `--aim` starts that game directly as before.

| Option | Effect |
|---|---|
| `--menu boot\|main\|connecting\|lobby\|create\|loading` | show that screen (screenshots) |
| `--demo-rooms` | three sample rooms in the lobby while offline (one with a password) |
| `--menu-auto` | connects to `--region` and creates a default room without input |
| `--pause` (with a scene) | opens the in-game Tab menu on start |

Screenshots taken for this document (1920x1080, `--screenshot`): every `--menu` screen and `--pause`. `--menu-auto` was run
against the live servers (menu, Europe, lobby, create, fade, map with the player spawned, `MenuTest` in the kill feed).
`cargo test` covers the layout engine and the room-name rule.

# In-room menu, settings and PlayerPrefs

Spec of the CrazyGames build's (Unity 5.3.5, asm.js) in-room menu (`RoomMultiplayerMenu`), the settings it offers,
the scripts it drives (`EnableHelper`, `FPSMouseLook`) and every `PlayerPrefs` key the game uses.
Every statement is labelled **observed** (decompiled code or extracted data), **inferred** or **unknown**.

Evidence (decompiled with `tools/re/asmjs_func.py` into `research/notes/crazygames/`, gitignored):
`roommultiplayermenu.js` (`Start`, `Awake`, `Update`, `FixedUpdate`, `OnGUI`, `MainMenu`, `Resolutions`,
`QualityWindow`, `SpawnPlayer`, `SwapTeams`, `LeaveRoom`), `fpsmouselook.js`, `enablehelper.js`, `prefs_writers.js`
(`RagdollController.Start`, `WhoKilledWho.AddKillNotification`); `PlayerDamage.SwapTeams`/`DoSwapTeams` and
`WhoKilledWho.PlayerJoinedTeam` (printed with the same tool). Data: `assets/data/settings/QualitySettings.json`,
`AudioManager.json`, the `__Room` node of every map scene, the skin
`assets/data/scriptable/UnityEngine.GUISkin/room swat 1__resources.assets_40924.json`, the player prefabs.
Code: `room_menu.rs` (the menu), `prefs.rs` (PlayerPrefs and session settings), `imgui.rs` (`GUILayout` widgets shared
with the lobby), `player.rs` (`FPSMouseLook`), `main.rs` (wiring, `EnableHelper`, spawning).

## What exists and what does not (observed)

| Setting | In the original | Where |
|---|---|---|
| Mouse sensitivity | yes: 0.1 .. 5.1 in steps of 0.1, saved | Tab menu, PlayerPrefs `sens` |
| Graphics quality | yes: the six `QualitySettings` levels, **not saved** | Tab menu |
| Resolution / fullscreen | yes: the `Screen.resolutions` list, P key | Tab menu, P |
| Controls | a fixed list of key texts (no rebinding) | Tab menu, "Controls" tab |
| Volume / mute | **no**: no code calls `AudioListener` or sets an `AudioSource` volume | (client-only setting, below) |
| Language | **no**: every text is an English literal; nothing reads `Application.systemLanguage` | |

The only other `PlayerPrefs` users are `ConnectMenu` (`docs/gameplay/menu.md`), `RagdollController`, `WhoKilledWho`
and PUN's best-region cache (unused: the client always connects to a chosen region).
A second settings script, `MainMenu` (FPS Kit single-player, with its own `Resolutions`/`QualityWindow`), is attached to
no scene or prefab and needs no port.

## PlayerPrefs (observed)

| Key | Type | Read | Written |
|---|---|---|---|
| `PlayerName` | string | `ConnectMenu.Start` (if `HasKey`) | joining / creating a room, "Create Room" |
| `Clan` | string | `ConnectMenu.Start` | same |
| `GamePlays` | int | `ConnectMenu.Start` | `ConnectMenu.Start`: `+1` per lobby start |
| `Kills` | int | `ConnectMenu` (level, "Kills:" box) | `WhoKilledWho.AddKillNotification`: `SetInt("Kills", GetInt("Kills") + 1)` per kill by this client |
| `Deaths` | int | `ConnectMenu` ("Deaths:" box) | `RagdollController.Start`: when `gui && cam` (the owner's own death prefab), `SetInt("Deaths", GetInt("Deaths") + 1)` |
| `sens` | float | `RoomMultiplayerMenu.Start` (if `HasKey`) | the sensitivity `<` / `>` buttons |

- `AddKillNotification` compares `PhotonNetwork.playerName` with `PhotonNetwork.player.name` (always equal), reads
  `Kills` into the static `WhoKilledWho.Kills`, and writes `Kills + 1`.
- `RagdollController.Start` stores `GetInt("Deaths")` in the static `RagdollController.Deaths` and writes it `+ 1`.
  `gui` is set by `RespawnAfter`, which only the owner sends, so other players' death prefabs do not count.
  A team swap (`DoSwapTeams`, below) also instantiates the owner's death prefab with `RespawnAfter`, so **swapping teams
  counts as a death** in PlayerPrefs (not in the `Deaths` player property).
- Typed storage: `GetInt` of a key saved as float (or the other way) returns the default (Unity's documented
  behaviour, **inferred** for this build).
- Unity WebGL keeps PlayerPrefs in the browser's IndexedDB. **Client:** a JSON file
  `$XDG_DATA_HOME/pga3-client/prefs.json` (fallback `~/.local/share/...`); ints are written as JSON integers and floats
  with a decimal point, so the type survives a reload. Every `Set*` writes the file at once (Unity flushes on
  quit or `Save()`; writing at once only differs after a crash).

## RoomMultiplayerMenu (the `__Room` object of every map)

Fields from the scene (**observed**): `guiSkin` = "room swat 1" (`resources.assets:40924`), `roomCamera` = the map's
`Camera` (`level1:2` on Bridge), `blackScreen` = `textures/BlackScreen.png`, `updateRate` 5 (ctor).
Statics: `sens` = **1.8** (`.cctor`), `teamnumb1`, `teamnumb2` (team sizes as last counted by the player list).
The ctor sets `sensfix = "1.8"` and `finalText = "......"`.

### Awake / Start (observed)

- `Awake`: `isMessageQueueRunning = true`; **`isPaused = true`** (the menu is open when the map loads);
  `resolutions = Screen.resolutions`, `resolutionIndex = (resolutions.Length - 1) / 2`;
  `QualityNames = QualitySettings.names`; `playerList = true`; `enableHelper = FindWithTag("EnableHelper")`; team
  names `German-ZM` / `Mercenary`, colours `Color.yellow` / `Color.red`; the player and room properties
  (`docs/crazygames-build.md`).
- `Start`: `if (PlayerPrefs.HasKey("sens")) sens = PlayerPrefs.GetFloat("sens")`; `ConnectMenu`'s static at +8 = 1.

### Update (observed)

1. `SuperWeapons` from the page URL (irrelevant here).
2. `CFInput.GetKeyDown(Tab)`: `isPaused = !isPaused`.
3. `Input.GetKeyDown(P)`: `Screen.fullScreen = !Screen.fullScreen`; then, if `Screen.fullScreen` is false,
   `Screen.SetResolution(resolutions[resolutionIndex], true)`. The setter takes effect at the end of the frame, so the
   getter still returns the old value: going **to** fullscreen applies the selected resolution, leaving it does not
   (**inferred** from Unity's deferred `fullScreen`).
4. Not paused: `enableHelper.SetActive(true)`, `Screen.lockCursor = true`. Paused: `SetActive(false)`, `lockCursor = false`.
5. Every `updateRate` (5 s): player property `Ping = PhotonNetwork.GetPing()` (already in `online.rs`).

### FixedUpdate (observed)

- While paused, or always in DM: `allPlayers = PhotonNetwork.playerList` sorted by `SortPlayers`
  (`B.Kills - A.Kills`, 0 when either has no `Kills`: descending kills) and `leadingPlayer` = the first name.
  `List.Sort` is not stable; **client:** stable sort, ties by actor number (**inferred**).
- Round timer: unless `delayAction`, `currentRoundTime = max(0, roundDuration - (PhotonNetwork.time - referenceTime))`;
  at 0 the `RoundEnded` coroutine starts (round end is not ported: `docs/unknowns.md` #32).
- `team1Score` / `team2Score` from the room properties.

### OnGUI (observed)

`GUI.skin = guiSkin`, then in order:

1. `GUI.color = black`; `GUI.Label(Rect(W-160, H-80, 190, 20), " [Tab - MENU]")` and
   `GUI.Label(Rect(W-160, H-100, 190, 20), " [P -FullScreen on/off]")`.
2. `GUI.color = (1, 1, 1, 0.7)`.
3. If paused: `servnamefix` = room name without `[WEP01]`..`[WEP10]`;
   `GUI.Window(0, Rect(W/2-370, H/2-237, 750, 470), MainMenu, "Room: " + servnamefix minus its last 16 characters)`.
4. Round time: `t = CeilToInt(currentRoundTime)`, `GUI.Box(Rect(W/2-50, 45, 100, 30),
   String.Format("{0:00}:{1:00}", FloorToInt((t / 60) % 60), FloorToInt(t % 60)))` (still at alpha 0.7).
5. TDM only: `BeginArea(Rect(15, 40, 200, 30))`, `BeginHorizontal("box", Width 200, Height 30)`, team-1 colour,
   `Label(team1 + ": ")`, `Space(5)`, white, `Label(team1Score)`, ends; then `BeginArea(Rect(15, 80, 200, 30))`,
   `BeginHorizontal("box", Width 0, Height 0)`, team-2 colour, `Label(team2 + ":")`, `Space(5)`, white,
   `Label(team2Score)`, ends.
6. White; if `roundEnded`: `Box(Rect(W/2-200, H/2-100, 400, 30), finalText)`.
7. `FadeScreen` (only while leaving: `fadeDir == 1`).

`GUI.Window` draws its frame and runs `MainMenu` after every script's `OnGUI` (windows are on top). The window keeps the
`GUI.color` of the `Window` call (alpha 0.7) for its frame and as the starting colour of `MainMenu` (Unity's
`GUIWindow` stores colour, skin and enabled state; **inferred** from memory of Unity's source). `MainMenu` calls
`GUI.FocusWindow(0)`, so the frame is drawn in the style's **on** state (background `SpellSlot_Type0_HoverOverlay.png`;
**inferred**: focused windows draw `onNormal`).

### MainMenu (the window, observed)

`GUILayout` inside the window (the window style's padding (0, 15, -5, 0) is the layout area):

```
Space(10)
BeginHorizontal
  Resolutions():  BeginVertical; Label("Resolution");
                  BeginScrollView(scroll, Width 140, Height 100); BeginVertical
                    per resolution: GUI.color = (0,20,20,0.6) if it equals Screen.width x height, else (20,20,20,0.6)
                                    Button("<w> x <h>") -> resolutionIndex = i; if Screen.fullScreen: SetResolution(w, h, true)
                  EndVertical; EndScrollView; EndVertical
  QualityWindow(): BeginVertical; white; Label("Quality");
                  BeginScrollView(scroll2, Width 240, Height 110); BeginVertical
                    per level: GUI.color = (0,20,20,0.6) if it is the current level, else (20,20,20,0.6)
                               Button(name) -> QualitySettings.SetQualityLevel(i, true)
                  EndVertical; EndScrollView; EndVertical
  white; Space(15)
  BeginVertical
    TDM: Button("German-ZM " + teamnumb1), Button("Mercenary " + teamnumb2)   (GUI.enabled per rule below)
    DM:  Button("Join")   enabled iff no Player and TeamName == "Spectators"
    GUI.enabled = true
    if Player exists: Button("Resume") -> isPaused = false
    Button("Leave Room") -> LeaveRoom
  EndVertical
EndHorizontal
Space(3)
BeginHorizontal
  Label("Mouse Sensitivity:" + sensfix + "%")
  Button("<", Width 30, Height 30) -> if sens > 0.1:  sens -= 0.1; SetFloat("sens", sens)
  Button(">", Width 30, Height 30) -> if sens < 5.0:  sens += 0.1; SetFloat("sens", sens)
EndHorizontal
Space(3); Space(10)
BeginHorizontal
  GUI.color = playerList ? (0,20,0,0.6) : white;   Button("Player List", Width 150, Height 25) -> playerList = true
  GUI.color = playerList ? white : (0,20,0,0.6);   Button("Controls",    Width 150, Height 25) -> playerList = false
EndHorizontal
Space(5); white
BeginScrollView(scroll3, Width 670, Height 300)
  Controls: GUI.color = (20,20,0,0.6); one Label each: "Tab - Pause Menu", "P - Fullscreen", "T - Chat / Enter - send",
            "C - crouch", "Left Ctrl - prone", "LMB - fire", "RMB - aim", "R - reload", "Left Shift - run",
            "Space - jump", "Buttons 1,2,3,4,5,6,7,8 - weapon change"
  Player List: GUI.color = (1,1,1,0.8); BeginHorizontal; FlexibleSpace
    TDM: team-1 colour; Label(team1); FlexibleSpace; EndHorizontal; teamnumb1 = 0; rows of TeamName == team1 (teamnumb1++ each)
         BeginHorizontal; FlexibleSpace; team-2 colour; Label(team2); FlexibleSpace; EndHorizontal; teamnumb2 = 0; rows of team2
    DM:  green; Label("All Players"); FlexibleSpace; EndHorizontal; rows of TeamName != "Spectators"
    BeginHorizontal; FlexibleSpace; green; Label("- Spectators -"); FlexibleSpace; EndHorizontal
    spectator rows: GUI.color green (this client) or white; BeginHorizontal("box"); Label(name); FlexibleSpace;
                    Label("Ping: " + Ping) if set; EndHorizontal
EndScrollView
```

- **Row** (team or DM): `GUI.color` = green if the row's name equals this client's name, else white;
  `BeginHorizontal("box")`; `Label(name, Width 150)`; `Label("Kills: " + Kills, Width 115)`;
  `Label("Deaths: " + Deaths, Width 115)`; `FlexibleSpace`; `Label("Ping: " + Ping)` if the property exists;
  `EndHorizontal`. (Several calls here go through the exception-wrapper thunks `Bc`/`Xa`/`md`/`Tb`; their targets
  follow from the arguments and the identical non-wrapped code nearby: **inferred**.)
- **Team-button rule** (team 1; team 2 mirrored): enabled iff `TeamName != team1` **and** `teamnumb1 <= teamnumb2` **and**
  (a Player exists **or** `TeamName == "Spectators"`). The counts come from the last player-list pass: they are only
  recounted while the player list is shown.
- **Team-button click:** Player exists ? `SwapTeams(team)` : `SpawnPlayer(team)`; then `isPaused = false` and
  `SendMessage("PlayerJoinedTeam", team)` (DM's "Join": `team_1`'s name, message argument `"battle"`).
  `WhoKilledWho.PlayerJoinedTeam(t)` sends `networkAddMessage(name, "", "joined " + t, "")` to all.
- **sensfix:** a chain of 51 tests `sens.ToString("0.0") == c.ToString("0.0")` for c = 0.1, 0.2, ..., 5.1 picks the label
  text. The values are `2 × round(c × 10)` written with two digits ("02", ..., "24") except **1.3 → "25"** and
  **5.1 → "100"** (5.0 → "98"); a value matching none keeps the previous text (initially "1.8"). Default 1.8 shows
  "Mouse Sensitivity:34%".
- The `±0.1` steps are float (`sens` drifts: 1.8 + 32 × 0.1 in float is just under 5, so `>` still works there and
  5.1 is reachable). The client does the same `f32` arithmetic.
- `GUI.color` components above 1 multiply the style colours and are clamped to [0, 1] per vertex (IMGUI vertices
  are 8-bit colours; **inferred**). So (0,20,20) turns a grey button cyan and the red button text black; (0,20,0)
  turns it green; (20,20,0) turns the green label text yellow.

### SpawnPlayer / SwapTeams (observed)

- `SpawnPlayer(team)`: destroy the current Player (if any); `enableHelper.SetActive(true)`; `TeamName = team`;
  team 1: in TDM a random team-1 spawn point, in DM a random point of both lists, prefab `playerPrefab`; otherwise a
  random team-2 point and `playerPrefabTwo`; `PhotonNetwork.Instantiate`; object named after the player;
  **`roomCamera.SetActive(false)`** (never re-activated). Until the first spawn the map is seen through the room camera.
- `SwapTeams(team)`: `TeamName = team`; `Player.SendMessage("SwapTeams")` → `PlayerDamage.SwapTeams`:
  `RpcSecure("DoSwapTeams", All, encrypt false)`. `DoSwapTeams` on every client: instantiate the death prefab
  (`ragdoll`) at the player's transform; the owner sends it `RespawnAfter` and runs `DestroyPlayer(0)`; the others send
  `clearCamera`. After the respawn countdown the owner's `SpawnPlayer` uses the new team. The player's children are not
  hidden first (unlike a death).
- `LeaveRoom`: `PhotonNetwork.LeaveRoom()` when connected (`OnLeftRoom` loads `Main`: `docs/gameplay/menu.md`).

## EnableHelper (both player prefabs, observed)

`Start`: `enablerReferenceObject = FindWithTag("EnableHelper")` (the empty `EnableHelper` object of every map).
`FixedUpdate`, with the room menu switching that object in its `Update`:

- object active: `fpsController.canControl = true`; if the `WeaponManager` and its `SelectedWeapon` (a `WeaponScript`)
  are both disabled, enable both; if both `FPSMouseLook`s are disabled, enable both;
- object inactive (menu open): the reverse (`canControl = false`, both weapon behaviours disabled, both mouse looks
  disabled);
- re-finds the object when it is gone.

So the open menu stops movement input (`canControl`), weapon switching and firing (`WeaponManager.Update`/`OnGUI` and
`WeaponScript.Update`; coroutines keep running) and mouse look, from the next physics step on. The game itself keeps
running (no `timeScale` change). (`docs/parity-checklist.md` row 6 named this `FPScontroller.enabled`; the code writes
the byte at offset 0x10, `canControl`.)

## FPSMouseLook (CrazyGames body, observed)

Prefab values: root `German`/`Mercenarys` axes 1, `sensitivityX` 4; `LookObject` axes 2, `sensitivityY` 3.5,
`minimumY`/`maximumY` -80/80 (the ctor's 15/15/±80 are overwritten by the prefab).

`Update` (skipped while `Time.timeScale < 0.01`):

1. axes 1: `Rotate(0, MouseX · sensitivityX, 0)`; axes 2: `rotationY = clamp(rotationY + MouseY · sensitivityY,
   min, max)`, `localEulerAngles = (-rotationY, y, 0)` (axes 0, both, is unused).
2. `weapScript`: if `weapManager.SelectedWeapon` exists, keep the cached script or cache
   `SelectedWeapon.GetComponent<WeaponScript>()`; if there is no selected weapon, clear it.
3. `currentSensitivity = RoomMultiplayerMenu.sens × (weapScript.aimed ? 0.25 : 1)`;
   `sensitivityX = sensitivityY = currentSensitivity`.

So a sensitivity change applies from the **next** frame, the very first frame turns with the prefab's 4 / 3.5, and
the aim factor follows the **first** selected weapon's script, which stays cached (inactive weapons are not destroyed);
the first weapon (the Machete) never aims, so in practice the factor never applies. Same as Build-7.
`Mouse X/Y` scale: `docs/gameplay/movement.md` (0.1 per pixel, `docs/unknowns.md` #8).

## Quality levels (observed data, engine effects inferred)

`QualitySettings.names`: Fastest, Fast, Simple, **Good** (the build's `m_CurrentQuality` 3, the level a player starts
with), Beautiful, Fantastic. `SetQualityLevel(i, true)` applies at once; nothing saves it, so every session starts at
Good (Unity WebGL has no launcher dialog to remember it: **inferred**).

| Level | textureQuality | pixelLightCount | shadows | lodBias | particleRaycastBudget |
|---|---|---|---|---|---|
| Fastest | 1 (half) | 0 | off | 0.3 | 4 |
| Fast | 2 (quarter) | 0 | off | 0.4 | 16 |
| Simple | 2 | 1 | off | 0.3 | 0 |
| Good | 1 | 1 | off | 0.3 | 0 |
| Beautiful | 0 (full) | 1 | off | 0.3 | 0 |
| Fantastic | 1 | 2 | hard | 0.3 | 0 |

All levels: `blendWeights` 1, no anti-aliasing, anisotropic off, `vSyncCount` 0, soft particles off.

- `textureQuality` (`masterTextureLimit`) skips that many top mip levels of every texture **with mipmaps** (291 of the
  397 textures, including the map materials and the menu background); textures without mipmaps are unaffected.
  So the default Good draws those textures at half resolution (**inferred**, `docs/unknowns.md` #38). **Client:** `GL_TEXTURE_BASE_LEVEL` =
  `min(limit, last level)` on mipmapped textures, applied to every loaded texture when the level changes.
- `shadows`: every map's directional light has soft shadows, so **Fantastic** would show realtime hard shadows
  (distance 40, one cascade). The client has no shadow maps: not ported (`docs/unknowns.md` #38).
- `pixelLightCount`: the maps have one light, a directional one, which Unity always renders per pixel: no effect.
- `lodBias`: the maps have no `LODGroup`: no effect. `particleRaycastBudget`: only limits particle collision of
  Medium/Low quality; whether any effect uses those is not checked (`docs/unknowns.md` #38).

## Resolutions and fullscreen

`Screen.resolutions` in Unity WebGL is **unknown** (`docs/unknowns.md` #38). **Client (inferred, standalone player
behaviour):** the current monitor's video modes, unique sizes, ascending; empty without a monitor list, in which case
the list shows the window size. P toggles fullscreen: going to fullscreen uses the selected video mode
(exclusive fullscreen, borderless where the platform has no exclusive mode, e.g. Wayland); clicking a resolution
while fullscreen switches to it.

## Volume (client-only setting)

The original has none: the browser tab's volume is the only control. `AudioManager.m_Volume` = 1.0 is the project's
`AudioListener.volume`; `audio.rs` multiplies the mix by it. On top, the client keeps its own master volume
(0 .. 1, default 1) in PlayerPrefs under the client-only key `pga3-client.volume` (a name the original never uses),
set with `--volume V` (saved) or by editing `prefs.json`. There is no menu control, so the Tab menu stays as in
the original.

## Client

- The game started from the menu begins as the original: menu open, no player, the room camera, "Spectators";
  a team button spawns the player and sends "joined <team>". Starting a scene directly (`--online`, `--dummy`,
  a scene name) still spawns at once with `--team` (test runs); `--pause` opens the menu on such a game.
- `--menu-auto` presses the `--team` button by itself.
- Tab toggles the menu; Esc only releases the mouse (in the browser Esc leaves pointer lock; it does not open the menu).
- The player list shows this client, the `--dummy` client and, online, every actor with its custom properties
  (`TeamName`, `Kills`, `Deaths`, `Ping`) as the server reports them.
- `--quality N` sets the level for the session (as `QualityWindow` would; not saved). `--volume V` sets and saves the
  client volume. With `--menu-auto` the game takes the `--screenshot` (after `--wait`) instead of the menu.
- The room skin gives the horizontal scrollbar no height and the client draws none; a list wider than the window
  scrolls only vertically (`docs/unknowns.md` #37).
- A `prefs.json` after a few games:

  ```json
  {
      "Clan": "",
      "Deaths": 3,
      "GamePlays": 4,
      "Kills": 12,
      "PlayerName": "Player",
      "pga3-client.volume": 0.5,
      "sens": 2.3
  }
  ```
- Not ported: the round end (`RoundEnded`, `Restart`, `finalText`), the leave fade (`FadeScreen` with `fadeDir == 1`),
  realtime shadows of Fantastic.

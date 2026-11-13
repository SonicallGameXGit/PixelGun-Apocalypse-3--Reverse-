//! `RoomMultiplayerMenu`'s `OnGUI` and its window (`MainMenu`): the Tab menu of a room with the
//! resolution and quality lists, the team / join / resume / leave buttons, the mouse
//! sensitivity, the player list and the controls (spec: `docs/gameplay/settings.md`).
//!
//! The game (`main.rs`) owns what the buttons change; this module draws and returns the action.

use glam::{Vec2, Vec4};

use crate::gui::{Gui, GuiFont, NamedSkin};
use crate::imgui::{self, Builder, DrawState, Frame};
use crate::prefs::QualityLevel;
use crate::room::Team;

/// `RoomMultiplayerMenu.guiSkin` ("room swat 1").
pub const ROOM_SKIN: &str = "resources.assets:40924";
/// `TeamName` of a player who has not chosen a team.
pub const SPECTATORS: &str = "Spectators";
/// `WaitBeforeCheck`: the round timer starts 0.3 s after `Awake`.
pub const ROUND_DELAY: f32 = 0.3;
const SCROLL_STEP: f32 = 40.0;

const CONTROLS: [&str; 11] = [
    "Tab - Pause Menu",
    "P - Fullscreen",
    "T - Chat / Enter - send",
    "C - crouch",
    "Left Ctrl - prone",
    "LMB - fire",
    "RMB - aim",
    "R - reload",
    "Left Shift - run",
    "Space - jump",
    "Buttons 1,2,3,4,5,6,7,8 - weapon change",
];

/// `GUI.color` of a selected list entry and of the others (components above 1 saturate).
const SELECTED: Vec4 = Vec4::new(0.0, 20.0, 20.0, 0.6);
const UNSELECTED: Vec4 = Vec4::new(20.0, 20.0, 20.0, 0.6);
const TAB_SELECTED: Vec4 = Vec4::new(0.0, 20.0, 0.0, 0.6);
const CONTROLS_COLOR: Vec4 = Vec4::new(20.0, 20.0, 0.0, 0.6);
const GREEN: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);

#[derive(Clone, Debug, PartialEq)]
pub enum RoomAction {
    Resolution(usize),
    Quality(usize),
    /// A team button (1 or 2) or, in DM, "Join" (team 1).
    Team(u8),
    Resume,
    Leave,
    SensitivityDown,
    SensitivityUp,
    PlayerList,
    Controls,
}

/// The mouse events of one frame on the menu.
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseEvents {
    pub position: Vec2,
    /// Left button went down / up this frame.
    pub down: bool,
    pub up: bool,
    pub wheel: f32,
}

/// A `PhotonPlayer` with the custom properties the menu shows (`None`: not set).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerRow {
    pub name: String,
    pub team: Option<String>,
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub ping: Option<i32>,
}

/// What `OnGUI` reads from the room and the game this frame.
pub struct RoomView<'a> {
    pub size: (f32, f32),
    /// The room's full name (with weapon tags and the marker).
    pub room_name: &'a str,
    pub game_mode: &'a str,
    pub team_1: &'a Team,
    pub team_2: &'a Team,
    pub team1_score: i32,
    pub team2_score: i32,
    pub round_time: f32,
    /// `PhotonNetwork.playerList`.
    pub players: &'a [PlayerRow],
    pub local_name: &'a str,
    pub local_team: &'a str,
    /// `Player != null`: this client's player object exists.
    pub player_exists: bool,
    pub resolutions: &'a [(u32, u32)],
    pub quality_levels: &'a [QualityLevel],
    pub quality: usize,
    pub sens: f32,
}

pub struct RoomMenu {
    /// `isPaused`: the window is open (true from `Awake` on).
    pub paused: bool,
    /// `playerList`: the list tab is shown, else the controls.
    pub player_list: bool,
    /// `resolutionIndex`.
    pub resolution_index: usize,
    /// `sensfix`: the percentage text of the sensitivity label.
    sensfix: String,
    /// The statics `teamnumb1` / `teamnumb2`: team sizes as the last player-list pass counted.
    teamnumb: [i32; 2],
    /// `scroll`, `scroll2`, `scroll3`.
    scroll: [f32; 3],
    pressed: Option<RoomAction>,
    skin: NamedSkin,
}

/// `string.Format("{0:00}:{1:00}", minutes, seconds)` of `CeilToInt(currentRoundTime)`.
pub fn round_text(round_time: f32) -> String {
    let t = round_time.ceil() as i32;
    let seconds = t % 60;
    let minutes = (t / 60) % 60;
    format!("{minutes:02}:{seconds:02}")
}

/// `servnamefix`: the room name without `[WEP01]`..`[WEP10]`, then the window title cuts its
/// last 16 characters (the `PixelWarfare001v` marker).
pub fn window_title(room_name: &str) -> String {
    let mut text = room_name.to_string();
    for n in 1..=10 {
        text = text.replace(&format!("[WEP{n:02}]"), "");
    }
    let keep = text.chars().count().saturating_sub(16);
    let name: String = text.chars().take(keep).collect();
    format!("Room: {name}")
}

/// The `sensfix` chain: `sens.ToString("0.0")` compared with 0.1, 0.2, ..., 5.1; `None` when
/// nothing matches (the text stays).
pub fn sensitivity_text(sens: f32) -> Option<String> {
    let tenths = (sens * 10.0).round() as i32;
    if format!("{sens:.1}") != format!("{:.1}", tenths as f32 / 10.0) {
        return None;
    }
    match tenths {
        13 => Some("25".to_string()),
        1..=12 => Some(format!("{:02}", tenths * 2)),
        14..=51 => Some(format!("{:02}", tenths * 2 - 2)),
        _ => None,
    }
}

/// `sens` after `<` / `>` (float steps, as the original adds `-0.1f` / `0.1f`).
pub fn step_sensitivity(sens: f32, up: bool) -> Option<f32> {
    if up {
        (sens < 5.0).then_some(sens + 0.1)
    } else {
        (sens > 0.1).then_some(sens + -0.1)
    }
}

impl RoomMenu {
    pub fn new(skin: NamedSkin, resolutions: usize) -> RoomMenu {
        RoomMenu {
            paused: true,
            player_list: true,
            resolution_index: resolutions.saturating_sub(1) / 2,
            sensfix: "1.8".to_string(),
            teamnumb: [0; 2],
            scroll: [0.0; 3],
            pressed: None,
            skin,
        }
    }

    fn area(size: (f32, f32)) -> [f32; 4] {
        [size.0 / 2.0 - 370.0, size.1 / 2.0 - 237.0, 750.0, 470.0]
    }

    /// `MainMenu(windowID)` as `GUILayout` calls.
    fn build(
        &mut self,
        gui: &mut Gui,
        font: &mut GuiFont,
        view: &RoomView,
        bars: [f32; 3],
    ) -> Frame<RoomAction> {
        let window = self.skin.style("window");
        let area = RoomMenu::area(view.size);
        let mut b = Builder::new(gui, font, &self.skin, area, window.padding);
        // The window function starts with the colour of the `GUI.Window` call.
        b.color = Vec4::new(1.0, 1.0, 1.0, 0.7);
        b.space(10.0);
        b.horizontal();
        // Resolutions().
        b.vertical();
        b.label("Resolution", None, None);
        b.begin_scroll((140.0, 100.0), 140.0 - bars[0]);
        b.vertical();
        let (w, h) = (view.size.0 as u32, view.size.1 as u32);
        for (i, &(rw, rh)) in view.resolutions.iter().enumerate() {
            b.color = if (rw, rh) == (w, h) {
                SELECTED
            } else {
                UNSELECTED
            };
            b.button(
                &format!("{rw} x {rh}"),
                RoomAction::Resolution(i),
                None,
                None,
            );
        }
        b.end();
        b.end_scroll();
        b.end();
        // QualityWindow().
        b.vertical();
        b.color = Vec4::ONE;
        b.label("Quality", None, None);
        b.begin_scroll((240.0, 110.0), 240.0 - bars[1]);
        b.vertical();
        for (i, level) in view.quality_levels.iter().enumerate() {
            b.color = if i == view.quality {
                SELECTED
            } else {
                UNSELECTED
            };
            b.button(&level.name, RoomAction::Quality(i), None, None);
        }
        b.end();
        b.end_scroll();
        b.end();
        b.color = Vec4::ONE;
        b.space(15.0);
        b.vertical();
        let spectator = view.local_team == SPECTATORS;
        if view.game_mode == "TDM" {
            let [n1, n2] = self.teamnumb;
            b.enabled = view.local_team != view.team_1.name
                && n1 <= n2
                && (view.player_exists || spectator);
            b.button(
                &format!("{} {n1}", view.team_1.name),
                RoomAction::Team(1),
                None,
                None,
            );
            b.enabled = view.local_team != view.team_2.name
                && n2 <= n1
                && (view.player_exists || spectator);
            b.button(
                &format!("{} {n2}", view.team_2.name),
                RoomAction::Team(2),
                None,
                None,
            );
        } else {
            b.enabled = !view.player_exists && spectator;
            b.button("Join", RoomAction::Team(1), None, None);
        }
        b.enabled = true;
        if view.player_exists {
            b.button("Resume", RoomAction::Resume, None, None);
        }
        b.button("Leave Room", RoomAction::Leave, None, None);
        b.end();
        b.end();
        b.space(3.0);
        b.horizontal();
        if let Some(text) = sensitivity_text(view.sens) {
            self.sensfix = text;
        }
        b.label(&format!("Mouse Sensitivity:{}%", self.sensfix), None, None);
        b.button("<", RoomAction::SensitivityDown, Some(30.0), Some(30.0));
        b.button(">", RoomAction::SensitivityUp, Some(30.0), Some(30.0));
        b.end();
        b.space(3.0);
        b.space(10.0);
        b.horizontal();
        b.color = if self.player_list {
            TAB_SELECTED
        } else {
            Vec4::ONE
        };
        b.button(
            "Player List",
            RoomAction::PlayerList,
            Some(150.0),
            Some(25.0),
        );
        b.color = if self.player_list {
            Vec4::ONE
        } else {
            TAB_SELECTED
        };
        b.button("Controls", RoomAction::Controls, Some(150.0), Some(25.0));
        b.end();
        b.space(5.0);
        b.color = Vec4::ONE;
        b.begin_scroll((670.0, 300.0), 670.0 - bars[2]);
        if self.player_list {
            self.teamnumb = player_list(&mut b, view, self.teamnumb);
        } else {
            b.color = CONTROLS_COLOR;
            for line in CONTROLS {
                b.label(line, None, None);
            }
        }
        b.end_scroll();
        b.finish()
    }

    /// The window laid out, with a scroll bar in each scroll view that overflows.
    fn frame(&mut self, gui: &mut Gui, font: &mut GuiFont, view: &RoomView) -> Frame<RoomAction> {
        let teamnumb = self.teamnumb;
        let frame = self.build(gui, font, view, [0.0; 3]);
        if !frame.any_overflow() {
            return frame;
        }
        let width = imgui::scrollbar_width(&self.skin);
        let mut bars = [0.0; 3];
        for (i, bar) in bars.iter_mut().enumerate() {
            if i < frame.scrolls.len() && frame.overflows(i) {
                *bar = width;
            }
        }
        // The counts of this frame come from one pass.
        self.teamnumb = teamnumb;
        self.build(gui, font, view, bars)
    }

    /// The mouse on the window (only while it is open): returns the clicked button's action
    /// (pressed and released on it).
    pub fn input(
        &mut self,
        gui: &mut Gui,
        font: &mut GuiFont,
        view: &RoomView,
        events: MouseEvents,
    ) -> Option<RoomAction> {
        let MouseEvents {
            position: mouse,
            down,
            up,
            wheel,
        } = events;
        if !self.paused {
            self.pressed = None;
            return None;
        }
        let frame = self.frame(gui, font, view);
        frame.scroll(&mut self.scroll, mouse, wheel, SCROLL_STEP);
        let hit = frame.hit(&self.scroll, mouse).cloned();
        if down {
            self.pressed = hit.clone();
        }
        if up {
            let pressed = self.pressed.take();
            if pressed.is_some() && pressed == hit {
                return hit;
            }
        }
        None
    }

    /// `OnGUI`: the key hints, the round time, the TDM scores and, while paused, the window
    /// (drawn last: windows are on top of every script's `OnGUI`).
    pub fn on_gui(&mut self, gui: &mut Gui, font: &mut GuiFont, view: &RoomView, mouse: Vec2) {
        let (w, h) = view.size;
        let label = self.skin.style("label");
        let boxed = self.skin.style("box");
        gui.begin_script();
        gui.color = Vec4::new(0.0, 0.0, 0.0, 1.0);
        gui.label(
            font,
            [w - 160.0, h - 80.0, 190.0, 20.0],
            " [Tab - MENU]",
            &label,
        );
        gui.label(
            font,
            [w - 160.0, h - 100.0, 190.0, 20.0],
            " [P -FullScreen on/off]",
            &label,
        );
        let window_color = Vec4::new(1.0, 1.0, 1.0, 0.7);
        gui.color = window_color;
        gui.box_(
            font,
            [w / 2.0 - 50.0, 45.0, 100.0, 30.0],
            &round_text(view.round_time),
            &boxed,
        );
        if view.game_mode == "TDM" {
            self.score_box(gui, font, view, 1);
            self.score_box(gui, font, view, 2);
        }
        if self.paused {
            let frame = self.frame(gui, font, view);
            gui.color = window_color;
            let style = self.skin.style("window");
            let area = RoomMenu::area(view.size);
            imgui::window_frame(gui, font, &style, area, &window_title(view.room_name), true);
            let from = gui.quads.len();
            let state = DrawState {
                mouse,
                pressed: self.pressed.as_ref(),
                focus: None,
                blink: 0.0,
                offsets: &self.scroll,
                alpha: 1.0,
                interactive: true,
            };
            imgui::draw(gui, font, &self.skin, &frame, &state);
            // `GUI.Window` clips its contents to the window.
            imgui::clip_quads(&mut gui.quads, from, area);
        }
        gui.color = Vec4::ONE;
    }

    /// `BeginArea(Rect(15, 40 | 80, 200, 30))` with a `BeginHorizontal("box")` holding
    /// "<team>: " and the score.
    fn score_box(&self, gui: &mut Gui, font: &mut GuiFont, view: &RoomView, team: u8) {
        let (y, size, t, score, colon) = if team == 1 {
            (40.0, Some(200.0), view.team_1, view.team1_score, ": ")
        } else {
            // Width(0), Height(0): the box draws nothing, the labels still show.
            (80.0, Some(0.0), view.team_2, view.team2_score, ":")
        };
        let mut b: Builder<RoomAction> =
            Builder::new(gui, font, &self.skin, [15.0, y, 200.0, 30.0], [0.0; 4]);
        b.color = Vec4::new(1.0, 1.0, 1.0, 0.7);
        let height = if team == 1 { Some(30.0) } else { Some(0.0) };
        b.horizontal_styled("box", size, height);
        b.color = t.color;
        b.label(&format!("{}{colon}", t.name), None, None);
        b.space(5.0);
        b.color = Vec4::ONE;
        b.label(&score.to_string(), None, None);
        b.end();
        let frame = b.finish();
        let state = DrawState {
            mouse: Vec2::splat(-1.0),
            pressed: None,
            focus: None,
            blink: 0.0,
            offsets: &[],
            alpha: 1.0,
            interactive: false,
        };
        imgui::draw(gui, font, &self.skin, &frame, &state);
    }

    /// Applies the tab buttons (the rest is the game's).
    pub fn select_tab(&mut self, action: &RoomAction) {
        match action {
            RoomAction::PlayerList => self.player_list = true,
            RoomAction::Controls => self.player_list = false,
            _ => {}
        }
    }
}

/// The player list: a header and rows per team (TDM) or all players (DM), then the spectators.
/// Returns the new `teamnumb1` / `teamnumb2`.
fn player_list(b: &mut Builder<RoomAction>, view: &RoomView, mut counts: [i32; 2]) -> [i32; 2] {
    let header = |b: &mut Builder<RoomAction>, color: Vec4, text: &str| {
        b.horizontal();
        b.flexible_space();
        b.color = color;
        b.label(text, None, None);
        b.flexible_space();
        b.end();
    };
    b.color = Vec4::new(1.0, 1.0, 1.0, 0.8);
    if view.game_mode == "TDM" {
        for (i, team) in [view.team_1, view.team_2].into_iter().enumerate() {
            header(b, team.color, &team.name);
            counts[i] = 0;
            for p in view.players {
                if p.team.as_deref() == Some(team.name.as_str()) {
                    counts[i] += 1;
                    player_row(b, view, p);
                }
            }
        }
    } else {
        header(b, GREEN, "All Players");
        for p in view.players {
            if p.team.as_deref() != Some(SPECTATORS) {
                player_row(b, view, p);
            }
        }
    }
    header(b, GREEN, "- Spectators -");
    for p in view.players {
        if p.team.as_deref() == Some(SPECTATORS) {
            b.color = row_color(view, p);
            b.horizontal_box();
            b.label(&p.name, None, None);
            b.flexible_space();
            if let Some(ping) = p.ping {
                b.label(&format!("Ping: {ping}"), None, None);
            }
            b.end();
        }
    }
    counts
}

fn row_color(view: &RoomView, p: &PlayerRow) -> Vec4 {
    if p.name == view.local_name {
        GREEN
    } else {
        Vec4::ONE
    }
}

/// `"Kills: " + customProperties["Kills"]`: a missing property concatenates as nothing.
fn property(value: Option<i32>) -> String {
    value.map_or(String::new(), |v| v.to_string())
}

fn player_row(b: &mut Builder<RoomAction>, view: &RoomView, p: &PlayerRow) {
    b.color = row_color(view, p);
    b.horizontal_box();
    b.label(&p.name, Some(150.0), None);
    b.label(&format!("Kills: {}", property(p.kills)), Some(115.0), None);
    b.label(
        &format!("Deaths: {}", property(p.deaths)),
        Some(115.0),
        None,
    );
    b.flexible_space();
    if let Some(ping) = p.ping {
        b.label(&format!("Ping: {ping}"), None, None);
    }
    b.end();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitivity_labels_follow_the_chain() {
        assert_eq!(sensitivity_text(1.8).as_deref(), Some("34"));
        assert_eq!(sensitivity_text(0.1).as_deref(), Some("02"));
        assert_eq!(sensitivity_text(1.2).as_deref(), Some("24"));
        assert_eq!(sensitivity_text(1.3).as_deref(), Some("25"));
        assert_eq!(sensitivity_text(1.4).as_deref(), Some("26"));
        assert_eq!(sensitivity_text(5.0).as_deref(), Some("98"));
        assert_eq!(sensitivity_text(5.1).as_deref(), Some("100"));
        assert_eq!(sensitivity_text(5.2), None);
        assert_eq!(sensitivity_text(0.0), None);
        // Float steps from the default reach 5.1 (the last `>` starts just under 5).
        let mut sens = 1.8f32;
        let mut steps = 0;
        while let Some(next) = step_sensitivity(sens, true) {
            sens = next;
            steps += 1;
        }
        assert_eq!(steps, 33);
        assert_eq!(sensitivity_text(sens).as_deref(), Some("100"));
        let mut sens = 1.8f32;
        while let Some(next) = step_sensitivity(sens, false) {
            sens = next;
        }
        assert_eq!(sensitivity_text(sens).as_deref(), Some("02"));
    }

    #[test]
    fn round_time_and_title() {
        assert_eq!(round_text(1200.0), "20:00");
        assert_eq!(round_text(59.2), "01:00");
        assert_eq!(round_text(0.0), "00:00");
        assert_eq!(round_text(3725.0), "02:05");
        let name = "Room 12: TDM[WEP02][WEP03][WEP10]PixelWarfare001v";
        assert_eq!(window_title(name), "Room: Room 12: TDM");
    }
}

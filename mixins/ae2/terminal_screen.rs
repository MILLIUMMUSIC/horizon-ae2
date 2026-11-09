// SPDX-License-Identifier: LGPL-3.0-or-later
// AbstractTerminalPart.onUseWithoutItem opens the terminal menu on the server
// when the hand is empty. The control `screen` command reports `container`
// only after the client applies a supported OpenScreen. Assigning the server
// Game screen does not: the guest action restores it, and lan_open_container
// rejects ae2:cable_bus, so the window stays on `playing`.
//
// shortcut: minecraft:generic_9x1 is the OpenScreen the client draws as
// `container`. Replace it with MEStorageMenu when that menu can be shown.
// shortcut: native_open_menu needs the hub, which is not on Game during the
// guest right-click. request() holds the menu until the level tick sends it.

/// Side-part id of the ME terminal.
pub const TERMINAL: &str = "ae2:terminal";
/// Menu `native_open_menu` can send. Rows select `minecraft:generic_9x{rows}`.
pub const MENU_ID: &str = "minecraft:generic_9x1";
pub const MENU_ROWS: u8 = 1;
/// `gui.ae2.Terminal` in the pinned language file.
pub const TITLE_KEY: &str = "gui.ae2.Terminal";

const FACES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
const OFFSETS: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

/// The clicked block face, in the same order cable NBT stores side parts.
pub fn face_name(face: [i32; 3]) -> Option<&'static str> {
    OFFSETS
        .iter()
        .zip(FACES)
        .find(|(offset, _)| **offset == face)
        .map(|(_, name)| name)
}

/// Holding `ae2:terminal` against an empty cable face stores that one side part.
pub fn places(hand: Option<&str>, face: &str, occupied: bool) -> bool {
    hand == Some(TERMINAL) && FACES.contains(&face) && !occupied
}

/// One stored side part. `grid_node` is the part's `gn` tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part<'a> {
    pub id: &'a str,
    pub grid_node: bool,
}

/// What the server sends so the client leaves `playing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Menu {
    pub id: &'static str,
    pub rows: u8,
    pub title_key: &'static str,
}

/// Screen names the control `screen` command reports for this click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Playing,
    Container,
}

impl Screen {
    pub fn label(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Container => "container",
        }
    }
}

/// Empty-hand use of the single terminal stored on `face`.
/// Other faces, missing nodes, and a second part on the same face do not open.
pub fn open(hand_empty: bool, face: &str, parts: &[(&str, Part<'_>)]) -> Option<Menu> {
    if !hand_empty || !FACES.contains(&face) {
        return None;
    }
    let mut found = None;
    for (side, part) in parts {
        if *side != face {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(*part);
    }
    let part = found?;
    if part.id != TERMINAL || !part.grid_node {
        return None;
    }
    Some(Menu {
        id: MENU_ID,
        rows: MENU_ROWS,
        title_key: TITLE_KEY,
    })
}

/// `playing` stays until a menu is requested; that request is a container screen.
pub fn show(screen: Screen, menu: Option<Menu>) -> Screen {
    if menu.is_some() {
        Screen::Container
    } else {
        screen
    }
}

/// Menu waiting for `native_open_menu` once the server hub is back on Game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    pub uuid: u128,
    pub rows: u8,
    pub title_key: &'static str,
}

static PENDING: std::sync::Mutex<Option<Pending>> = std::sync::Mutex::new(None);

pub fn request(uuid: u128, menu: Menu) {
    *PENDING.lock().unwrap_or_else(|error| error.into_inner()) = Some(Pending {
        uuid,
        rows: menu.rows,
        title_key: menu.title_key,
    });
}

pub fn take() -> Option<Pending> {
    PENDING
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take()
}

static PLACED_AT: std::sync::Mutex<Option<u64>> = std::sync::Mutex::new(None);

/// A placement click can be followed by another use in the same tick after the
/// item leaves the hand. That use is not the empty-hand open.
pub fn note_placed(tick: u64) {
    *PLACED_AT.lock().unwrap_or_else(|error| error.into_inner()) = Some(tick);
}

pub fn placed_this_click(tick: u64) -> bool {
    PLACED_AT
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .is_some_and(|placed| tick.saturating_sub(placed) <= 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal() -> Part<'static> {
        Part {
            id: TERMINAL,
            grid_node: true,
        }
    }

    #[test]
    fn empty_hand_on_the_stored_terminal_leaves_playing() {
        let parts = [("north", terminal())];
        let menu = open(true, "north", &parts);
        assert_eq!(
            menu,
            Some(Menu {
                id: MENU_ID,
                rows: MENU_ROWS,
                title_key: TITLE_KEY,
            })
        );
        assert_eq!(show(Screen::Playing, menu).label(), "container");
        assert_eq!(parts, [("north", terminal())]);
    }

    #[test]
    fn a_click_that_does_not_open_stays_on_playing() {
        let stored = [("north", terminal())];
        let stayed = [
            open(false, "north", &stored),
            open(true, "north", &[]),
            open(true, "up", &stored),
            open(
                true,
                "north",
                &[(
                    "north",
                    Part {
                        id: TERMINAL,
                        grid_node: false,
                    },
                )],
            ),
            open(
                true,
                "north",
                &[(
                    "north",
                    Part {
                        id: "ae2:facade",
                        grid_node: true,
                    },
                )],
            ),
            open(
                true,
                "north",
                &[(
                    "north",
                    Part {
                        id: "ae2:crafting_terminal",
                        grid_node: true,
                    },
                )],
            ),
            open(true, "side", &[("side", terminal())]),
            open(
                true,
                "north",
                &[("north", terminal()), ("north", terminal())],
            ),
        ];
        for menu in stayed {
            assert_eq!(menu, None);
            assert_eq!(show(Screen::Playing, menu).label(), "playing");
        }
    }

    #[test]
    fn the_cable_center_does_not_add_a_part_or_block_the_terminal() {
        let parts = [
            (
                "cable",
                Part {
                    id: "ae2:fluix_glass_cable",
                    grid_node: true,
                },
            ),
            ("north", terminal()),
        ];
        let menu = open(true, "north", &parts);
        assert!(menu.is_some());
        assert_eq!(parts.len(), 2);
        assert_eq!(
            parts.iter().filter(|(_, part)| part.id == TERMINAL).count(),
            1
        );
    }

    #[test]
    fn the_north_face_places_only_on_an_empty_side() {
        assert_eq!(face_name([0, 0, -1]), Some("north"));
        assert!(places(Some(TERMINAL), "north", false));
        assert!(!places(Some(TERMINAL), "north", true));
        assert!(!places(None, "north", false));
        assert!(!places(Some("ae2:fluix_glass_cable"), "north", false));
    }

    #[test]
    fn the_queued_menu_is_sent_once() {
        let menu = Menu {
            id: MENU_ID,
            rows: MENU_ROWS,
            title_key: TITLE_KEY,
        };
        request(7, menu);
        assert_eq!(
            take(),
            Some(Pending {
                uuid: 7,
                rows: MENU_ROWS,
                title_key: TITLE_KEY,
            })
        );
        assert_eq!(take(), None);
        note_placed(10);
        assert!(placed_this_click(12));
        assert!(!placed_this_click(13));
    }
}

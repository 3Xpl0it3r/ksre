use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use super::action::{Action, RouteId};

struct KeyAction {
    action: Action,
    route_id: RouteId,
}

/* fn keybinds() {
    let a = HashMap::from([(
        KeyCode::Char('j'),
        KeyAction {
            action: Action::NOP,
            route_id: RouteId::PodLog,
        },
    )]);
} */


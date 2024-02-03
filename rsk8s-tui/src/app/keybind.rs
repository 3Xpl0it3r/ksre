use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use super::action::{Action, RouteId};

struct KeyContext {
    action: Action,
    route_id: RouteId,
}

/*
 *
 * keybinds should be as
 * {
 *      route_id1: {
 *          'j' => action1,
 *          'l' => action2,
 *          '<char>' => some_actions,
 *      },
 *      route_id2 => {
 *          'j' = action3,
 *          'l' => action4,
 *      },
 *      route_id3 => {
 *          'j' => action5,
 *          'k' => action6,
 *      }
 *
 * }
 */


macro_rules! generate_keybindings {
    ($(($key:ident, $action:expr, $route_id:expr));*) => {
        pub struct KeyBindings;
        impl KeyBindings { $(pub const $key: KeyContext = KeyContext{ action: $action, route_id: $route_id, };);* }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {}
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

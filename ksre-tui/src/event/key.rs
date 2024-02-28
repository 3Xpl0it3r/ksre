use std::fmt::Debug;

use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub enum CusKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space,
    Backspace,
    Tab,
    Enter,
    Esc,
    None,
}

// Frm[#TODO] (should add some comments)
impl From<KeyEvent> for CusKey {
    fn from(value: KeyEvent) -> Self {
        match value.code {
            KeyCode::Char(c) => match c {
                'a' => CusKey::A,
                'b' => CusKey::B,
                'c' => CusKey::C,
                'd' => CusKey::D,
                'e' => CusKey::E,
                'f' => CusKey::F,
                'g' => CusKey::G,
                'h' => CusKey::H,
                'i' => CusKey::I,
                'j' => CusKey::J,
                'k' => CusKey::K,
                'l' => CusKey::L,
                'm' => CusKey::M,
                'n' => CusKey::N,
                'o' => CusKey::O,
                'p' => CusKey::P,
                'q' => CusKey::Q,
                'r' => CusKey::R,
                's' => CusKey::S,
                't' => CusKey::T,
                'u' => CusKey::U,
                'v' => CusKey::V,
                'w' => CusKey::W,
                'x' => CusKey::X,
                'y' => CusKey::Y,
                'z' => CusKey::Z,
                ' ' => CusKey::Space,
                _ => CusKey::None,
            },
            KeyCode::Tab => CusKey::Tab,
            KeyCode::Enter => CusKey::Enter,
            KeyCode::Backspace => CusKey::Backspace,
            KeyCode::Esc => CusKey::Esc,
            _ => CusKey::None,
        }
    }
}

// CusKey[#TODO] (should add some comments)
impl CusKey {
    pub fn char(self) -> char {
        match self {
            CusKey::A => 'a',
            CusKey::B => 'b',
            CusKey::C => 'c',
            CusKey::D => 'd',
            CusKey::E => 'e',
            CusKey::F => 'f',
            CusKey::G => 'g',
            CusKey::H => 'h',
            CusKey::I => 'i',
            CusKey::J => 'j',
            CusKey::K => 'k',
            CusKey::L => 'l',
            CusKey::M => 'm',
            CusKey::N => 'n',
            CusKey::O => 'o',
            CusKey::P => 'p',
            CusKey::Q => 'q',
            CusKey::R => 'r',
            CusKey::S => 's',
            CusKey::T => 't',
            CusKey::U => 'u',
            CusKey::V => 'v',
            CusKey::W => 'w',
            CusKey::X => 'x',
            CusKey::Y => 'y',
            CusKey::Z => 'z',
            CusKey::Space => ' ',
            CusKey::Tab => '_',
            CusKey::Enter => ';',
            CusKey::None => '-',
            CusKey::Backspace => ' ',
            CusKey::Esc => '~',
        }
    }
}

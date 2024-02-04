use std::cmp::{self, Ordering};
use std::usize;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::event::Event;

pub struct Context {
    tab: RouteId,
    mode: Mode,
}

const ROUTE_STEP: isize = 100;

#[derive(Clone, Copy, PartialEq)]
pub enum RouteId {
    PodIndex = 0,
    PodInpu,
    PodList,
    PodSecInfo,
    PodStatus,
    PodYaml,
    PodLog,
    PodTerminal,
    // ...gf.
    PodEnd,
    DeployIndex = ROUTE_STEP,
    // ...
    DeployEnd,
    NodeIndex = 2 * ROUTE_STEP,
    NodeEnd,
}

// Ord[#TODO] (should add some comments)
impl PartialOrd for RouteId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this = *self as isize;
        let another = *other as isize;
        Some(this.cmp(&another))
    }
}

impl RouteId {
    pub fn next(self) -> Self {
        if self as usize == 200 {
            RouteId::PodIndex
        } else {
            let c_tb_nr = self as usize;
            match c_tb_nr {
                0 => RouteId::DeployIndex,
                100 => RouteId::NodeIndex,
                _ => unreachable!(),
            }
        }
    }
}

// endit mode
#[derive(Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
    // in command module will disable all key event from tui terminal
    // when live command mode, appstate will empty all event buffer from tui event channel
    Command,
}

#[derive(Debug)]
pub enum Action {}

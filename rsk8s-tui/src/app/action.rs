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
    // ....
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
pub enum Action {
    TabNext,
    // 有数据更新刷新页面
    Resync,
    // 时钟中断
    Tick,

    StsItemNext,
    StsItemPrev,
    // pod相关操作
    PodExec,
    PodLogs,
    // 退出操作
    Quit,
    // 没有任何操作
    NOP,
}

pub fn get_action(event: &Event, tb_nr: RouteId) -> Action {
    match tb_nr {
        RouteId::PodIndex => get_action_for_tb_pod(event),
        _ => match event {
            Event::Tick => Action::NOP,
            Event::Key(key) => match key.code {
                KeyCode::Tab => Action::TabNext,
                KeyCode::Char(c) => match c {
                    'q' => Action::Quit,
                    _ => Action::NOP,
                },
                _ => Action::NOP,
            },
            _ => Action::NOP,
        },
    }
}

fn get_action_for_tb_pod(event: &Event) -> Action {
    match event {
        Event::Tick => Action::Tick,
        Event::Key(key) => match key.code {
            KeyCode::Tab => Action::TabNext,
            KeyCode::Char(c) => match c {
                'q' => Action::Quit,
                'j' => Action::StsItemNext,
                'k' => Action::StsItemPrev,
                'l' => Action::PodLogs,
                't' => Action::PodExec,
                _ => Action::NOP,
            },
            _ => Action::NOP,
        },
        Event::Error => Action::NOP,
        _ => Action::Resync, //针对pod相关的操作意味着已经更新缓存了,因此只需要重新刷新页面就行
    }
}

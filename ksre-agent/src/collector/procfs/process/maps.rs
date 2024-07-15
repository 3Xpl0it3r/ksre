use super::Pid;

#[derive(Debug)]
pub struct Maps {}

impl Maps {
    fn read(_pid: Pid) -> Self {
        unreachable!("impl this")
    }
}

// From<Pid>[#TODO] (should add some comments)
impl From<Pid> for Maps {
    fn from(pid: Pid) -> Self {
        Maps::read(pid)
    }
}

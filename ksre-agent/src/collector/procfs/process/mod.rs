use ksre_lib::serializer::bytes::BytesCodec;

use self::io::IO;
use self::stack::Stack;
use self::status::Status;

mod io;
mod maps;
mod smaps;
mod stack;
mod stat;
mod status;
mod syscall;

mod constant;

pub type Pid = u64;

// ProcessState[#TODO] (shoule add some comments )
#[derive(Debug)]
pub struct ProcessState {
    io: io::IO,
    stack: stack::Stack,
    status: status::Status,
    /* smaps: smaps::Smaps, */
    /* maps: maps::Maps,
    stat: stat::Stat,
    status: status::Status,
    syscall: syscall::Syscall, */
}

// ProcessState[#TODO] (should add some comments)
impl ProcessState {
    pub fn new(_pid: u64) -> Self {
        Self {
            io: IO::mock(),
            stack: Stack::mock(),
            status: Status::mock(),
            /* smaps: smaps::Smaps::(), */
            /* maps: pid.into(),
            samps: pid.into(),
            stat: pid.into(),
            status: pid.into(),
            syscall: pid.into(), */
        }
    }
}

// BytesCodec[#TODO] (should add some comments)
impl BytesCodec for ProcessState {
    fn byte_encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // encode io
        buffer.extend(self.io.byte_encode());
        // encode stack
        buffer.extend(self.stack.byte_encode());
        // encode status
        buffer.extend(self.status.byte_encode());

        buffer
    }

    fn byte_decode(&mut self, buffer: &[u8]) -> usize {
        let mut offset = 0;
        offset += self.io.byte_decode(&buffer[offset..]);
        offset += self.stack.byte_decode(&buffer[offset..]);
        offset += self.status.byte_decode(&buffer[offset..]);

        offset
    }
}

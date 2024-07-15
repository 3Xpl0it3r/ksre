use std::fs::{self};

use ksre_lib::serializer::bytes::BytesCodec;

use super::Pid;

#[derive(Default, Debug)]
pub struct Stack {
    stacks: Vec<String>,
}

impl Stack {
    fn read(pid: Pid) -> Self {
        let content = fs::read_to_string(format!("/proc/{pid}/stack")).unwrap();
        let mut stack = Stack::default();
        stack.read_from(&content);
        stack
    }

    #[inline]
    fn read_from(&mut self, content: &str) {
        let mut lines = content
            .split('\n')
            .map(|line| {
                let item = line.split(' ').collect::<Vec<&str>>();
                item[1].to_string()
            })
            .collect::<Vec<String>>();
        lines.reverse();
        lines.remove(0);
        self.stacks = lines;
    }
}

// Stack[#TODO] (should add some comments)
impl Stack {
    pub fn mock() -> Self {
        let expectd = vec![
            "el0t_64_sync+0x1a0/0x1a4".to_string(),
            "el0t_64_sync_handler+0x13c/0x1c4".to_string(),
            "el0_svc+0x90/0xc0".to_string(),
            "do_el0_svc+0xe0/0x128".to_string(),
            "invoke_syscall.constprop.0+0x88/0xd8".to_string(),
            "__arm64_sys_futex+0x17c/0x2d4".to_string(),
            "do_futex+0xdc/0x8e4".to_string(),
            "futex_wait+0xdc/0x1cc".to_string(),
            "futex_wait_queue_me+0xbc/0x110".to_string(),
            "__switch_to+0xc8/0xe0".to_string(),
        ];
        Self { stacks: expectd }
    }
}

// From<Pid>[#TODO] (should add some comments)
impl From<Pid> for Stack {
    fn from(pid: Pid) -> Self {
        Stack::read(pid)
    }
}

// BytesCodec[#TODO] (should add some comments)
impl BytesCodec for Stack {
    fn byte_encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // write number of stack message into buffer
        buffer.extend((self.stacks.len() as u64).byte_encode());
        for stack in self.stacks.iter() {
            buffer.extend(stack.byte_encode());
        }
        buffer
    }

    fn byte_decode(&mut self, buffer: &[u8]) -> usize {
        let mut stack_len: u64 = 0;
        let mut offset = 0;
        let mut readed = stack_len.byte_decode(buffer);
        offset += readed;
        for _ in 0..stack_len {
            let mut message = String::new();
            readed = message.byte_decode(&buffer[offset..]);
            offset += readed;
            self.stacks.push(message);
        }
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let conent = r"[<0>] __switch_to+0xc8/0xe0
[<0>] futex_wait_queue_me+0xbc/0x110
[<0>] futex_wait+0xdc/0x1cc
[<0>] do_futex+0xdc/0x8e4
[<0>] __arm64_sys_futex+0x17c/0x2d4
[<0>] invoke_syscall.constprop.0+0x88/0xd8
[<0>] do_el0_svc+0xe0/0x128
[<0>] el0_svc+0x90/0xc0
[<0>] el0t_64_sync_handler+0x13c/0x1c4
[<0>] el0t_64_sync+0x1a0/0x1a4
[<ffffffffffffffff>] 0xffffffffffffffff";
        let mut stack = Stack::default();
        stack.read_from(conent);

        let expectd = vec![
            "el0t_64_sync+0x1a0/0x1a4".to_string(),
            "el0t_64_sync_handler+0x13c/0x1c4".to_string(),
            "el0_svc+0x90/0xc0".to_string(),
            "do_el0_svc+0xe0/0x128".to_string(),
            "invoke_syscall.constprop.0+0x88/0xd8".to_string(),
            "__arm64_sys_futex+0x17c/0x2d4".to_string(),
            "do_futex+0xdc/0x8e4".to_string(),
            "futex_wait+0xdc/0x1cc".to_string(),
            "futex_wait_queue_me+0xbc/0x110".to_string(),
            "__switch_to+0xc8/0xe0".to_string(),
        ];

        assert_eq!(expectd, stack.stacks);
    }
}

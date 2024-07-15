#[derive(Default, Debug)]
pub struct Meta {
    pub freelist_page: u64,
    pub root: u64,
}

impl Meta {
    pub fn deserialize(&mut self, buffer: &[u8]) {
        let mut offset = 0;
        self.root = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;

        self.freelist_page = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // this code is ommit the warnning by compiler
        _ = offset;
    }

    pub fn serialize(&self, buffer: &mut [u8]) {
        let mut offset = 0;

        buffer[offset..offset + 8].clone_from_slice(u64::to_le_bytes(self.root).as_ref());
        offset += 8;

        buffer[offset..offset + 8].clone_from_slice(u64::to_le_bytes(self.freelist_page).as_ref());
        offset += 8;
        // for extend
        //
        // this code is ommit the warnning by compiler
        _ = offset;
    }
}

#[cfg(test)]
mod tests {}

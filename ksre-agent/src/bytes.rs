pub struct Varint;

// VarintEncoder[#TODO] (should add some comments)
impl Varint {
    pub fn encode_u64(mut value: u64) -> Vec<u8> {
        let mut buffer = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;

            if value != 0 {
                byte |= 0x80; // 如果不是最后个字节，设置最高位
            }
            buffer.push(byte);

            if value == 0 {
                break;
            }
        }
        buffer
    }

    // return (count_readed, u64)
    pub fn read_u64(bytes: &[u8]) -> (usize, u64) {
        let mut value = 0;
        let mut shift = 0;
        let mut read_count = 0;

        for &byte in bytes {
            read_count += 1;
            let lower_7bits = (byte & 0x7F) as u64;
            value |= lower_7bits << shift;

            if byte & 0x80 == 0 {
                break;
            }

            shift += 7;
        }
        (read_count, value)
    }
}

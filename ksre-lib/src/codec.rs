pub trait Codec {
    fn encode(self) -> Vec<u8>;
    fn decode(&mut self, buffer: &[u8]);
}

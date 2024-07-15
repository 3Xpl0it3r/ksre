pub(super) trait VarintTyper {
    fn encode_varint(self) -> Vec<u8>;
    fn decode_varint(buffer: &[u8]) -> (usize, Self);
}

/* #[macro_export] */
macro_rules!  varint_impl_for {
    (@create $type_: ty) => {
        fn encode_varint(self) -> Vec<u8> {
            let mut value = self;
            let mut buffer = Vec::new();
            loop {
                let mut byte: u8 = (value & 0x7f) as u8;
                value >>= 7;

                if value != 0 {
                    byte |= 0x80;
                }
                buffer.push(byte);

                if value == 0 {
                    break;
                }
            }
            buffer
        }
        fn decode_varint( bytes: &[u8]) -> (usize, $type_) {
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
            (read_count, value as $type_)
        }
    };
    ($($type_ :ty),*) => {
        $(
            impl VarintTyper for $type_ {
              varint_impl_for!(@create $type_);
            }
        )*
    };
}

varint_impl_for!(u32, u64);

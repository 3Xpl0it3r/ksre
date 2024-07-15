pub trait BytesCodec {
    fn byte_encode(&self) -> Vec<u8>;
    fn byte_decode(&mut self, buffer: &[u8]) -> usize;
}

use crate::serializer::varint::VarintTyper;

macro_rules! bytes_codec_impl_for {
    (@gen_str $type_: ty) => {
        impl BytesCodec for $type_ {
            fn byte_encode(&self) -> Vec<u8> {
                let mut buffer = Vec::new();
                let key_size = self.as_bytes().len() as u64;
                buffer.extend(key_size.encode_varint());
                buffer.extend(self.as_bytes());
                buffer
            }
            fn byte_decode(&mut self, buffer: &[u8]) -> usize{
                let (readded, key_size) = u64::decode_varint(buffer);
                *self = String::from_utf8(buffer[readded..readded+key_size as usize].into()).unwrap();
                readded+key_size as usize
            }
        }
    };
    (@gen_num $type_: ty) => {
        impl BytesCodec for $type_ {
            fn byte_encode(&self) -> Vec<u8> {
                let mut buffer = Vec::new();
                buffer.extend(self.encode_varint());
                buffer
            }
            fn byte_decode(&mut self, buffer: &[u8]) -> usize {
                let (readed, value) = u64::decode_varint(buffer);
                *self = value;
                readed
            }
        }
    };
    (@as_type String) => {
        bytes_codec_impl_for!(@gen_str String);
    };
    (@as_type u32) => {
        bytes_codec_impl_for!(@gen_num u32);
    };
    (@as_type u64) => {
        bytes_codec_impl_for!(@gen_num u64);
    };

    ($($val: ident),*) => {
        $( bytes_codec_impl_for!(@as_type $val); )*
    }

}

bytes_codec_impl_for!(u64, String);

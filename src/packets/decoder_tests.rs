use super::*;

macro_rules! decoder_test {
    ($name: ident, $t: ty, $data: expr, $result: expr) => {
        #[test]
        fn $name() {
            let data = $data;
            {
                let mut reader = PacketReader::new(&data);
                let result = <$t as Decodable>::decode(&mut reader).unwrap();
                assert_eq!(reader.available(), 0); //We need to decode it all.
                assert_eq!(result, $result);
            }
        }
    }
}

decoder_test!(test_decode_i8_positive, i8,
[1u8],
1i8);

decoder_test!(test_decode_i8_negative, i8,
[253u8],
-3i8);

decoder_test!(test_decode_u8, u8,
[5u8], 5u8);

decoder_test!(test_decode_i16_positive, i16,
[0x12u8, 0x34u8], 0x1234i16);

decoder_test!(test_decode_i16_negative, i16,
[0xffu8, 0xfdu8], -3i16);

decoder_test!(test_decode_u16, u16,
[0x12u8, 0x34u8], 0x1234u16);

decoder_test!(test_decode_i32_positive, i32,
[0x12u8, 0x34u8, 0x56u8, 0x78u8], 0x12345678i32);

decoder_test!(test_decode_i32_negative, i32,
[0xffu8, 0xff, 0xff, 0xfd], -3i32);

decoder_test!(test_decode_u32, u32,
[0x12u8, 0x34, 0x56, 0x78], 0x12345678u32);

decoder_test!(test_decode_i64_positive, i64,
[0x12u8, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0],
0x123456789abcdef0i64);

decoder_test!(test_decode_i64_negative, i64,
[0xffu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd],
-3i64);

decoder_test!(test_decode_u64, u64,
[0x12u8, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0],
0x123456789abcdef0u64);

decoder_test!(test_decode_string, String,
[b'a', b' ', b't', b'e', b's', b't', 0],
"a test".to_string());

//Fastnet specific types:

decoder_test!(test_decode_fastnet_query, StatusRequest,
[0u8],
StatusRequest::FastnetQuery);

decoder_test!(test_decode_version_query, StatusRequest,
[1u8],
StatusRequest::VersionQuery);

decoder_test!(test_decode_extension_query, StatusRequest,
[2u8, b't', b'e', b's', b't', b'_', b'a', b't', b'e', b's', b't', 0],
StatusRequest::ExtensionQuery("test_atest".to_string()));

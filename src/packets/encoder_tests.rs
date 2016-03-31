use super::*;

macro_rules! encoder_test {
    ($name: ident, $result: expr, $($encodees: expr),* ) => {
        #[test]
        fn $name() {
            let mut array = [0u8;1024];
            {
                let mut dest = PacketWriter::new(&mut array);
                $(($encodees).encode(&mut dest).unwrap();)*
                assert!(dest.written() == ($result).len());
            }
            assert!($result[..] == array[..($result).len()]);
        }
    };
}

encoder_test!(test_encode_u8,
[0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9],
0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8);


encoder_test!(test_encode_i8,
[251u8, 252u8, 253u8, 254u8, 255u8, 0u8, 1u8, 2u8, 3u8, 4u8],
-5i8, -4i8, -3i8, -2i8, -1i8, 0i8, 1i8, 2i8, 3i8, 4i8);

encoder_test!(test_encode_u16,
[0u8, 1u8, 1u8, 0u8, 1u8, 1u8, 255u8, 255u8],
0x0001u16, 0x0100u16, 0x0101u16, 0xffffu16);

encoder_test!(test_encode_i16,
[0u8, 1, 0x23, 0x45, 0xff, 0xfe, 0xff, 0xff],
1i16, 0x2345i16, -2i16, -1i16);

encoder_test!(test_encode_u32,
[0u8, 0, 0, 1, 0x23, 0x45, 0x67, 0x89],
1u32, 0x23456789u32);

encoder_test!(test_encode_i32,
[0x23u8, 0x45, 0x67, 0x89, 0xff, 0xff, 0xff, 0xff],
0x23456789i32, -1i32);

encoder_test!(test_encode_u64,
[0u8, 0, 0, 0, 0, 0, 0, 5,
0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0],
5u64, 0x123456789abcdef0u64);

encoder_test!(test_encode_i64,
[0u8, 0, 0, 0, 0, 0, 0, 5,
0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe],
5i64, -2i64);

encoder_test!(test_encode_str,
[b'h', b'e', b'l', b'l', b'o', 0u8],
"hello");

encoder_test!(test_encode_string,
[b'h', b'e', b'l', b'l', b'o', 0u8],
"hello".to_string());

encoder_test!(test_encode_status_request,
[0u8, //Fastnet query.
1, //Version query.
2, b't', b'e', b's', b't', b'_',
b'a', b't', b'e', b's', b't', 0], //Extension query.
StatusRequest::FastnetQuery, StatusRequest::VersionQuery, StatusRequest::ExtensionQuery("test_atest".to_string()));

encoder_test!(test_encode_status_response,
[0u8, 1, //fastnet response.
1, b'1', b'.', b'0', 0, //Version.
2, b't', b'e', b's', b't', b'_',
b'a', b't', b'e', b's', b't', 0, 1], //Extension "test_atest" is supported.
StatusResponse::FastnetResponse(1), StatusResponse::VersionResponse("1.0".to_string()),
StatusResponse::ExtensionResponse{name: "test_atest".to_string(), supported: true});

//We assume that primitive types are tested sufficiently by the above.
//So test one variant each of the Packet enum, using the simplest inner representations we can for testing.

encoder_test!(test_encode_status_request_packet,
[255u8, 255, 0, 0], //status request of type fastnet query.
Packet::StatusRequest(StatusRequest::FastnetQuery));

encoder_test!(test_encode_status_response_packet,
[255, 255, 1, 0, 1], //Fastnet is listening.
Packet::StatusResponse(StatusResponse::FastnetResponse(1)));

encoder_test!(test_encode_connect_packet,
[255, 255, 2], //Request for connection.
Packet::Connect);

encoder_test!(test_encode_connected_packet,
[255, 255, 3, 0, 0, 0, 5], //Connected, id is 5.
Packet::Connected(5));

encoder_test!(test_encode_aborted_packet,
[255, 255, 4, b'f', b'a', b'i', b'l', 0], //aborted with message "fail".
Packet::Aborted("fail".to_string()));

encoder_test!(test_encode_heartbeat_packet,
[255, 254, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 15],
Packet::Heartbeat{counter: 5, sent: 10, received: 15});

encoder_test!(test_encode_echo_packet,
[255u8, 253, 0, 5],
Packet::Echo(5));

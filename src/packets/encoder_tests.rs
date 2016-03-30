use super::encoder::*;

#[test]
fn test_encode_u8() {
    let mut array = [0u8; 10];
    {
        let mut dest = PacketWriter::new(&mut array);
        for x in 0u8..10u8 {
            x.encode(&mut dest).unwrap();
        }
        assert!(dest.available() == 0); //we should have filled it.
    }
    assert!(array == [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_encode_i8() {
    let mut array = [0u8; 10];
    {
        let mut dest = PacketWriter::new(&mut array);
        for x in -5i8..5i8 {
            x.encode(&mut dest).unwrap();
        }
        assert!(dest.available() == 0);
    }
    //The following literal comes from knowledge of twos complement arithmetic.
    //namely, if x>0 then -x = (~x)+1
    assert!(array == [251u8, 252u8, 253u8, 254u8, 255u8, 0u8, 1u8, 2u8, 3u8, 4u8]);
}

#[test]
fn test_encode_u16() {
    let mut array = [0u8; 8]; //encoding 4 u16s.
    {
        let mut dest = PacketWriter::new(&mut array);
        1u16.encode(&mut dest).unwrap();
        256u16.encode(&mut dest).unwrap();
        0x1010u16.encode(&mut dest).unwrap();
        0xffffu16.encode(&mut dest).unwrap();
        assert!(dest.available() == 0);
    }
    assert!(array == [0u8, 1u8, 1u8, 0u8, 0x10u8, 0x10u8, 0xffu8, 0xffu8]);
}

#[test]
fn test_encode_i16() {
    let mut array = [0u8; 8];
    {
        let mut dest = PacketWriter::new(&mut array);
        1i16.encode(&mut dest).unwrap();
        0x2345i16.encode(&mut dest).unwrap();
        (-1i16).encode(&mut dest).unwrap();
        (-2i16).encode(&mut dest).unwrap();
        assert!(dest.available() == 0);
    }
    assert!(array == [0u8, 1, 0x23, 0x45, 0xff, 0xff, 0xff, 0xfe]);
}

#[test]
fn test_encode_u32() {
    let mut array = [0u8; 8];
    {
        let mut dest = PacketWriter::new(&mut array);
        1u32.encode(&mut dest).unwrap();
        0x23456789u32.encode(&mut dest).unwrap();
        assert!(dest.available() == 0);
    }
    assert!(array == [0u8, 0, 0, 1,
    0x23, 0x45, 0x67, 0x89]);
}

#[test]
fn test_encode_i32() {
    let mut array = [0u8; 8];
    {
        let mut dest = PacketWriter::new(&mut array);
        0x23456789i32.encode(&mut dest).unwrap();
        (-2i32).encode(&mut dest).unwrap();
        assert!(dest.available() == 0);
    }
    assert!(array == [0x23u8, 0x45, 0x67, 0x89,
    0xff, 0xff, 0xff, 0xfe]);
}

#[test]
fn test_encode_str() {
    let mut array = [0u8; 6];
    {
        let mut dest = PacketWriter::new(&mut array);
        "hello".encode(&mut dest).unwrap();
        assert!(dest.available() == 0);
    }
    assert!(array == [b'h', b'e', b'l', b'l', b'o', 0]);
}

#[test]
fn test_encode_string() {
    let mut array = [0u8; 6];
    {
        let mut dest = PacketWriter::new(&mut array);
        "hello".to_string().encode(&mut dest).unwrap();
        assert!(dest.available() == 0);
    }
    assert!(array == [b'h', b'e', b'l', b'l', b'o', 0]);
}
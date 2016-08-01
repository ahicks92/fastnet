use packets;
use std::iter;
use std::cmp;
use std::convert::From;
use std::borrow;

//This has to be small enough to leave room for the header.
const CHUNK_SIZE: usize = 500;

#[derive(Debug)]
pub struct FrameEncoder<'A, T: 'A> {
    channel: i16,
    sn: u64,
    last_reliable_frame: u64,
    reliable: bool,
    first: bool,
    iter: &'A mut T,
    workspace: Vec<u8>,
}

impl<'A, T, Q> FrameEncoder<'A, T> 
where T: iter::ExactSizeIterator+iter::Iterator<Item=Q>,
Q: borrow::Borrow<u8>,
Vec<u8>: iter::Extend<Q> {
    pub fn new(iter: &'A mut T, channel: i16, starting_sequence_number: u64, last_reliable_frame: u64, reliable: bool)->FrameEncoder<'A, T> {
        FrameEncoder {
            channel: channel,
            sn: starting_sequence_number,
            last_reliable_frame: last_reliable_frame,
            reliable: reliable,
            first: true,
            iter: iter,
            workspace: Vec::with_capacity(CHUNK_SIZE),
        }
    }
}

impl<'A, T, Q> iter::Iterator for FrameEncoder<'A, T>
where T: iter::ExactSizeIterator+iter::Iterator<Item=Q>,
Q: borrow::Borrow<u8>,
Vec<u8>: iter::Extend<Q> {
    type Item = packets::Packet;

    fn next(&mut self)->Option<packets::Packet> {
        let mut header = None;
        if self.first {
            let length = packets::FRAME_HEADER_SIZE+self.iter.len();
            header = Some(packets::FrameHeader{last_reliable_frame: self.last_reliable_frame, length: length as u32});
        }
        //Get some bytes.
        self.workspace.clear();
        self.workspace.extend(self.iter.take(CHUNK_SIZE));
        if self.first == false && self.workspace.len() == 0 {return None;}
        let dp = packets::DataPacketBuilder::with_payload_and_header(self.sn, self.workspace.clone(), header)
        .set_reliable(self.reliable)
        .set_frame_start(self.first)
        .set_frame_end(self.iter.len() == 0)
        .build();
        self.first = false;
        self.sn += 1;
        return Some(packets::Packet::Data{chan: self.channel, packet: dp});
    }
}


#[test]
fn test_frames() {
    let test_data: Vec<u8> = vec![1u8; 1000];
    //channel 100, start at sn 3, last reliable is 1.
    let mut got_packets = FrameEncoder::new(&mut test_data.iter(), 100, 3, 1, false).collect::<Vec<_>>();
    let mut expected_packets = vec![
        packets::Packet::Data{chan: 100,
            packet: packets::DataPacketBuilder::with_payload_and_header(3, vec![1u8; 500], Some(packets::FrameHeader{length: 1012, last_reliable_frame: 1})).build()
        },
        packets::Packet::Data{chan: 100,
            packet: packets::DataPacketBuilder::with_payload(4, vec![1u8; 500]).set_frame_end(true).build()
        }
    ];
    assert_eq!(got_packets, expected_packets);
    got_packets = FrameEncoder::new(&mut test_data.iter(), 100, 3, 1, true).collect();
    expected_packets = vec![
        packets::Packet::Data{chan: 100,
            packet: packets::DataPacketBuilder::with_payload_and_header(3, vec![1u8; 500], Some(packets::FrameHeader{length: 1012, last_reliable_frame: 1})).set_reliable(true).build()
        },
        packets::Packet::Data{chan: 100,
            packet: packets::DataPacketBuilder::with_payload(4, vec![1u8; 500]).set_reliable(true).set_frame_end(true).build(),
        }
    ];
    assert_eq!(got_packets, expected_packets);
}

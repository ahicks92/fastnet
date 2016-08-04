use std::collections;
use std::iter;
use packets;
use time;

const INITIAL_DURATION: u64 = 100000000;
const INITIAL_DURATION_MULTIPLIER: f64 = 1.1;

#[derive(Debug, Clone, PartialEq)]
struct AckRecord {
    packet: packets::Packet,
    next_time: u64,
    duration_multiplier: f64,
}

#[derive(Debug)]
pub struct AckManager {
    packets: collections::BTreeMap<(i16, u64), AckRecord>,
}

impl AckManager {
    pub fn new()->AckManager {
        AckManager {packets: collections::BTreeMap::default()}
    }

    /**Handles either ack or data.

Returns true if the packet was handled. Otherwise false.*/
    pub fn submit_packet(&mut self, packet: packets::Packet)->bool {
        let mut channel = 0i16;
        let mut sn = 0u64;
        match packet {
            packets::Packet::Ack{chan, sequence_number} => {
                //If in the map, kill it.
                self.packets.remove(&(chan, sequence_number));
                return true;
            },
            packets::Packet::Data{chan, packet: ref p} => {
                //To make this work out, we fall through after gathering the information we need.
                channel = chan;
                sn = p.sequence_number();
            },
            _ => {return false}
        }
        //If we get here, it's a data packet. Insert and return true.
        self.packets.insert((channel, sn), AckRecord{
            packet: packet,
            next_time: time::precise_time_ns()+INITIAL_DURATION,
            duration_multiplier: INITIAL_DURATION_MULTIPLIER,
        });
        return true;
    }

    pub fn iter_needs_ack<'A>(&'A mut self)->Box<iter::Iterator<Item=&'A packets::Packet>+'A> {
        let now = time::precise_time_ns();
        let mut res = self.packets.iter_mut().filter(move |i| {
            i.1.next_time <= now
        }).map(move |i| {
            let rec: &mut AckRecord  = i.1;
            rec.next_time = now+((INITIAL_DURATION as f64)*rec.duration_multiplier) as u64;
            rec.duration_multiplier *= INITIAL_DURATION_MULTIPLIER;
            &rec.packet
        });
        Box::new(res)
    }
}

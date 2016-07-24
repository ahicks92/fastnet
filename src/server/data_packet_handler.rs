use super::*;
use super::super::packets::*;
use super::super::async;
use super::super::constants;
use std::net;


/**handles acking packets, etc.*/
#[derive(Debug)]
pub struct DataPacketHandler {
    channel: i16,
    address : net::SocketAddr,
    ignore_number: u64,
    last_reliable_frame: u64,
    contained_payload: usize, //Used for cost limits.
    limit: usize, //the per-channel memory limit.
    acked_packets: Vec<DataPacket>,
    unacked_packets: Vec<DataPacket>,
}


impl DataPacketHandler {

    pub fn new(chan: i16, address: net::SocketAddr)->DataPacketHandler {
        DataPacketHandler {
            channel: chan,
            address: address,
            ignore_number: 0,
            last_reliable_frame: 0,
            contained_payload: 0,
            limit: constants::PER_CHANNEL_MEMORY_LIMIT_DEFAULT,
            acked_packets: Vec::default(),
            unacked_packets: Vec::default(),
        }
    }

    pub fn handle_incoming_packet<H: async::Handler>(&mut self, packet: DataPacket, service: &mut MioServiceProvider<H>) {
        let sn = packet.sequence_number();
        let reliable = packet.is_reliable();
        if(sn < self.ignore_number && reliable) {
            self.ack(sn, service);
            return;
        }
        else if(sn < self.ignore_number) {
            return;
        }
        if packet.is_reliable() {self.ensure_room(packet.borrow_payload().len());}
        let new_contained_payload = self.contained_payload + packet.borrow_payload().len();
        if(new_contained_payload > self.limit) {return;}
        //If the sequence nubmer is already in acked_packets then we ack and abort.
        //Otherwise, we just abort.
        let is_in_acked = self.acked_packets.binary_search_by_key(&packet.sequence_number(), |i| i.sequence_number());
        if let Ok(_) = is_in_acked {
            if(reliable) {self.ack(sn, service);}
            return;
        }
        let is_in_unacked = self.unacked_packets.binary_search_by_key(&packet.sequence_number(), |i| i.sequence_number());
        if let Ok(_) = is_in_unacked {return;}
        if(reliable) {
            self.unacked_packets.insert(is_in_unacked.unwrap_err(), packet);
        }
        else {
            self.acked_packets.insert(is_in_acked.unwrap_err(), packet);
        }
        self.contained_payload = new_contained_payload;
    }

    pub fn do_acks<H: async::Handler>(&mut self, service: &mut MioServiceProvider<H>) {
        //Because the acked packets are in order, failure to ack means we can stop early.
        let mut end_index = 0;
        for pack in self.unacked_packets.iter() {
            let sn = pack.sequence_number();
            if sn < self.ignore_number || sn == self.last_reliable_frame + 1 {
                self.ignore_number = sn;
                self.ack(sn, service);
                end_index += 1;
            }
            else {break;}
        }
        //Promote the packets.
        for pack in self.unacked_packets.drain(..end_index) {
            let ind = self.acked_packets.binary_search_by_key(&pack.sequence_number(), |i| i.sequence_number());
            if let Err(index) = ind {
                self.acked_packets.insert(index, pack);
            }
            //Otherwise it's a duplicate, so we do nothing.
        }
    }

    //Implements the packet dropping logic to allow incoming reliable packets to evict other, less important packets.
    pub fn ensure_room(&mut self, amount: usize) {
        if amount < self.limit-self.contained_payload {return;}
        let mut reliable_endpoint = self.unacked_packets.len();
        let mut unreliable_endpoint = 0;
        let mut sum = 0;
        //This simulates a goto, the best way to do the following.
        'outer: loop {
            for pack in self.acked_packets.iter() {
                if sum > amount {break 'outer;}
                if pack.is_reliable() == false {
                    sum += pack.borrow_payload().len();
                }
                unreliable_endpoint += 1;
            }
            for pack in self.unacked_packets.iter().rev() {
                if sum > amount {break 'outer;}
                if pack.is_reliable() {
                    sum += pack.borrow_payload().len();
                }
                reliable_endpoint -= 1;
            }
            break;
        }
        //Kill unreliables.
        for i in 0..unreliable_endpoint {
            if self.acked_packets[i].is_reliable() == false {
                self.acked_packets.remove(i);
            }
        }
        //The unacked packets are all reliable, so we can use drain.
        self.unacked_packets.drain(reliable_endpoint..);
        self.contained_payload -= sum;
    }

    pub fn ack<H: async::Handler>(&self, sn: u64, service: &mut MioServiceProvider<H>) {
        let packet = Packet::Ack{chan: self.channel, sequence_number: sn};
        service.send(packet, self.address);
    }

}

use super::*;
use super::super::async;
use std::time;
use std::collections;
use std::cmp;
use std::net;
use std::ops;
use uuid;
use super::super::packets::{Packet};


#[derive(Debug)]pub struct RoundtripEstimator {
    expected_echoes: collections::HashMap<uuid::Uuid, time::Instant>,
    estimation: Vec<u32>,
    required_echoes: usize,
    last_estimate: Option<u32>,
}

impl RoundtripEstimator {

    pub fn new(required_echoes: usize)->RoundtripEstimator {
        RoundtripEstimator {
            expected_echoes: collections::HashMap::default(),
            estimation: Vec::default(),
            required_echoes: required_echoes,
            last_estimate: None,
        }
    }

    pub fn get_last_estimate(&self)->Option<u32> {
        self.last_estimate
    }

    //Called once a second by established connections.
    pub fn tick<H: async::Handler>(&mut self, address: net::SocketAddr, endpoint_id: uuid::Uuid, service: &mut MioServiceProvider<H>) {
        let now = time::Instant::now();
        //Kill all echoes older than 5 seconds.
        let mut removing = Vec::with_capacity(self.expected_echoes.len());
        for i in self.expected_echoes.iter() {
            let dur = now.duration_since(*i.1);
            if dur.as_secs() >= 5 {
                removing.push(*i.0);
            }
        }
        for i in removing.iter() {
            self.expected_echoes.remove(i);
        }
        //Replace any if needed.
        if self.expected_echoes.len() < self.required_echoes {
            let needed_echoes = cmp::min(5, self.required_echoes-self.expected_echoes.len());
            for i in 0..needed_echoes {
                let uuid = uuid::Uuid::new_v4();
                self.expected_echoes.insert(uuid, now);
                service.send(Packet::Echo{endpoint: endpoint_id, uuid: uuid}, address);
            }
        }
    }

    pub fn handle_echo<H: async::Handler>(&mut self, connection_id: uuid::Uuid, echo_id: uuid::Uuid, service: &mut MioServiceProvider<H>) {
        if let Some(&instant) = self.expected_echoes.get(&echo_id) {
            let dur = time::Instant::now().duration_since(instant);
            let dur_ms: u64 = dur.as_secs()*1000+dur.subsec_nanos() as u64/1000000u64;
            self.estimation.push(dur_ms as u32);
            self.expected_echoes.remove(&echo_id);
        }
        if self.estimation.len() >= self.required_echoes {
            let average: u32 = self.estimation.iter().fold(0, ops::Add::add)/self.estimation.len() as u32;
            self.estimation.clear();
            service.handler.roundtrip_estimate(connection_id, average);
        }
    }
}

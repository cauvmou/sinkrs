use std::{fmt::Debug, net::{Ipv4Addr, Ipv6Addr}};

use super::{buffer::DnsPacketBuffer, string_to_bytes, util::Serialize};

#[derive(Debug, Clone)]
pub struct Record {
    pub cname: String,
    pub class: u16,
    pub ttl: u32,
    pub data: RecordData,
}

impl Record {
    pub fn new(buffer: &mut DnsPacketBuffer) -> Self {
        let cname = buffer.read_name();
        let rr_type = buffer.read_u16();
        let class = buffer.read_u16();
        let ttl = buffer.read_u32();
        let data_len = buffer.read_u16() as usize; 
        let data = RecordData::new(rr_type, data_len, buffer);
        Self {
            cname,
            class,
            ttl,
            data,
        }
    }
}

impl Into<Vec<u8>> for Record {
    fn into(self) -> Vec<u8> {
        [
            string_to_bytes(&self.cname), 
            [self.data.to_num().to_be_bytes(), self.class.to_be_bytes()].concat(), 
            self.ttl.to_be_bytes().to_vec(), self.data.serialize()
        ].concat()
    }
}

impl Serialize<Vec<u8>> for Record {}

#[derive(Debug, Clone)]
pub enum RecordData {
    UNKNOWN(Vec<u8>),
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
}

impl RecordData {
    pub fn new(rr_type: u16, len: usize, buffer: &mut DnsPacketBuffer) -> Self {
        match rr_type {
            1 => {Self::A(Ipv4Addr::new(buffer.read_u8(), buffer.read_u8(), buffer.read_u8(), buffer.read_u8()))},
            5 => {Self::CNAME(buffer.read_name())},
            28 => {Self::AAAA(Ipv6Addr::new(buffer.read_u16(), buffer.read_u16(), buffer.read_u16(), buffer.read_u16(), buffer.read_u16(), buffer.read_u16(), buffer.read_u16(), buffer.read_u16()))},
            _ => {Self::UNKNOWN(buffer.read_n(len))}
        }
    }

    pub fn to_num(&self) -> u16 {
        match self {
            RecordData::A(_) => 1,
            RecordData::AAAA(_) => 28,
            RecordData::CNAME(_) => 5,
            RecordData::UNKNOWN(_) => u16::MAX,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RecordData::UNKNOWN(data) => [(data.len() as u16).to_be_bytes().to_vec(), data.to_vec()].concat(),
            RecordData::A(ip) => [4u16.to_be_bytes().to_vec(), ip.octets().to_vec()].concat(),
            RecordData::AAAA(ip) => [16u16.to_be_bytes().to_vec(), ip.octets().to_vec()].concat(),
            RecordData::CNAME(name) => {
                let bytes = string_to_bytes(name);
                [(bytes.len() as u16).to_be_bytes().to_vec(), bytes].concat()
            },
        }
    }
}
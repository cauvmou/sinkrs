use self::{buffer::DnsPacketBuffer, record::{Record}, util::Serialize};

mod buffer;
pub mod record;
mod util;

#[derive(Debug, Clone, Copy)]
pub struct HeaderFlags {
    response: bool,
    opcode: u8,
    auth_answer: bool,
    truncated: bool,
    recursion_desired: bool,
    recursion_available: bool,
    z: u8,
    r_code: u8,
}

impl Serialize<u16> for HeaderFlags {}

impl From<u16> for HeaderFlags {
    fn from(value: u16) -> Self {
        Self {
            response: value & 0x8000 > 0,
            opcode: ((value >> 11) & 0xf) as u8,
            auth_answer: value & 0x400 > 0,
            truncated: value & 0x200 > 0,
            recursion_desired: value & 0x100 > 0,
            recursion_available: value & 0x80 > 0,
            z: ((value >> 4) & 0b111) as u8,
            r_code: (value & 0b1111) as u8,
        }
    }
}

impl Into<u16> for HeaderFlags {
    fn into(self) -> u16 {
        (self.response as u16) << 15
            | ((self.opcode as u16) << 11) as u16
            | (self.auth_answer as u16) << 10
            | (self.truncated as u16) << 9
            | (self.recursion_desired as u16) << 8
            | (self.recursion_available as u16) << 7
            | (self.z << 4) as u16
            | self.r_code as u16
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    transaction_id: u16,
    flags: HeaderFlags,
    questions: u16,
    answer_rrs: u16,
    authority_rrs: u16,
    additional_rrs: u16,
}

impl Into<Vec<u8>> for Header {
    fn into(self) -> Vec<u8> {
        [
            self.transaction_id.to_be_bytes(), 
            self.flags.serialize().to_be_bytes(), 
            self.questions.to_be_bytes(),
            self.answer_rrs.to_be_bytes(),
            self.authority_rrs.to_be_bytes(),
            self.additional_rrs.to_be_bytes(),
        ].concat()
    }
}

impl Serialize<Vec<u8>> for Header {}

#[derive(Debug, Clone)]
pub struct Question {
    pub cname: String,
    rr_type: u16,
    class: u16,
}

impl Into<Vec<u8>> for Question {
    fn into(self) -> Vec<u8> {
        [string_to_bytes(&self.cname), [self.rr_type.to_be_bytes().to_vec(), self.class.to_be_bytes().to_vec()].concat()].concat()
    }
}

impl Serialize<Vec<u8>> for Question {}

#[derive(Debug, Clone)]
pub struct DnsPacket {
    len: u16,
    header: Header,
    pub questions: Vec<Question>,
    pub records: Vec<Record>
}

impl<'a> From<&'a [u8]> for DnsPacket {
    fn from(value: &'a [u8]) -> Self {
        let mut buffer = DnsPacketBuffer::new(value);
        let header = Header {
            transaction_id: buffer.read_u16(),
            flags: buffer.read_u16().into(),
            questions: buffer.read_u16(),
            answer_rrs: buffer.read_u16(),
            authority_rrs: buffer.read_u16(),
            additional_rrs: buffer.read_u16(),
        };
        let questions = (0..header.questions)
            .map(|_| Question {
                cname: buffer.read_name(),
                rr_type: buffer.read_u16(),
                class: buffer.read_u16(),
            }).collect::<Vec<Question>>();
        let records = (0..header.answer_rrs)
            .map(|_| Record::new(&mut buffer)).collect::<Vec<Record>>();
        Self {
            len: buffer.position() as u16,
            header,
            questions,
            records,
        }
    }
}

fn string_to_bytes(string: &String) -> Vec<u8> {
    [string.split(".").map(|s| 
        [
            vec![s.len() as u8], 
            s.chars().into_iter().map(|c| c as u8).collect::<Vec<u8>>(),
        ].concat()
    ).collect::<Vec<Vec<u8>>>().concat(), vec![0]].concat()
}

impl Into<Vec<u8>> for DnsPacket {
    fn into(self) -> Vec<u8> {
        [
            self.header.serialize(), 
            self.questions.iter().map(|q| q.clone().serialize()).collect::<Vec<Vec<u8>>>().concat(), 
            self.records.iter().map(|a| a.clone().serialize()).collect::<Vec<Vec<u8>>>().concat()
        ].concat()
    }
}

impl DnsPacket {
    pub fn size(&self) -> Vec<u8> {
        self.len.to_be_bytes().to_vec()
    }

    pub fn from_tcp<'a>(bytes: &'a [u8], len: usize) -> Self {
        bytes[2..len].into()
    }

    pub fn bytes(self) -> Vec<u8> {
        let packet: Vec<u8> = self.into();
        [(packet.len() as u16).to_be_bytes().to_vec(), packet].concat()
    }
}

#[cfg(test)]
mod test {
    use super::{HeaderFlags};

    #[test]
    fn header_manual() {
        let packet = 0x8180u16;
        let flags = HeaderFlags { 
            response: true, opcode: 0, auth_answer: false, truncated: false, recursion_desired: true, recursion_available: true, z: 0, r_code: 0 
        };
        let serialized: u16 = flags.into();
        assert_eq!(serialized, packet);
    }

    #[test]
    fn header_serializing() {
        let packet = 0x8180u16;
        let header: HeaderFlags = packet.into();
        let serialized: u16 = header.into();
        assert_eq!(serialized, packet);
    }
}
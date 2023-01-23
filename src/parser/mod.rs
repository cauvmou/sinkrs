use self::buffer::DnsPacketBuffer;

mod buffer;

trait Serialize<T> where Self: Into<T>{
    #[inline]
    fn serialize(self) -> T {
        self.into()
    }
}

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
            auth_answer: value & 0x4000 > 0,
            truncated: value & 0x2000 > 0,
            recursion_desired: value & 0x1000 > 0,
            recursion_available: value & 0x800 > 0,
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
    name: String,
    rr_type: u16,
    class: u16,
}

impl Into<Vec<u8>> for Question {
    fn into(self) -> Vec<u8> {
        [string_to_bytes(&self.name), [self.rr_type.to_be_bytes(), self.class.to_be_bytes()].concat()].concat()
    }
}

impl Serialize<Vec<u8>> for Question {}

#[derive(Debug, Clone)]
pub struct Answer {
    name: String,
    rr_type: u16,
    class: u16,
    ttl: u32,
    rd_data: Vec<u8>
}

impl Into<Vec<u8>> for Answer {
    fn into(self) -> Vec<u8> {
        [string_to_bytes(&self.name), [self.rr_type.to_be_bytes(), self.class.to_be_bytes()].concat(), self.ttl.to_be_bytes().to_vec(), self.rd_data].concat()
    }
}

impl Serialize<Vec<u8>> for Answer {}

#[derive(Debug)]
pub struct DnsPacket {
    len: u16,
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>
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
                name: buffer.read_name(),
                rr_type: buffer.read_u16(),
                class: buffer.read_u16(),
            }).collect::<Vec<Question>>();
        let answers = (0..header.answer_rrs)
            .map(|_| Answer {
                name: buffer.read_name(),
                rr_type: buffer.read_u16(),
                class: buffer.read_u16(),
                ttl: buffer.read_u32(),
                rd_data: {let n = buffer.read_u16(); buffer.read_n(n as usize)},
            }).collect::<Vec<Answer>>();
        Self {
            len: buffer.position() as u16,
            header,
            questions,
            answers,
        }
    }
}

fn string_to_bytes(string: &String) -> Vec<u8> {
    string.split(".").map(|s| 
        [
            vec![s.len() as u8], 
            s.chars().into_iter().map(|c| c as u8).collect::<Vec<u8>>()
        ].concat()
    ).collect::<Vec<Vec<u8>>>().concat()
}

impl Into<Vec<u8>> for DnsPacket {
    fn into(self) -> Vec<u8> {
        [
            self.header.serialize(), 
            self.questions.iter().map(|q| q.clone().serialize()).collect::<Vec<Vec<u8>>>().concat(), 
            self.answers.iter().map(|a| a.clone().serialize()).collect::<Vec<Vec<u8>>>().concat()
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
}
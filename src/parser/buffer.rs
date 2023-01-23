#[derive(Clone, Copy)]
pub struct DnsPacketBuffer<'a> {
    buf: &'a [u8],
    ptr: usize,
}

impl<'a> DnsPacketBuffer<'a> {

    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            ptr: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.ptr
    }

    #[inline]
    pub fn peek_u8(&self) -> u8 {
        self.buf[self.ptr]
    }

    #[inline]
    pub fn peek_u16(&self) -> u16 {
        u16::from_be_bytes([self.buf[self.ptr], self.buf[self.ptr+1]])
    }

    #[inline]
    pub fn peek_u32(&self) -> u32 {
        u32::from_be_bytes([self.buf[self.ptr], self.buf[self.ptr+1], self.buf[self.ptr+2], self.buf[self.ptr+3]])
    }

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        (self.peek_u8(), {self.ptr += 1}).0
    }

    #[inline]
    pub fn read_u16(&mut self) -> u16 {
        (self.peek_u16(), {self.ptr += 2}).0
    }

    #[inline]
    pub fn read_u32(&mut self) -> u32 {
        (self.peek_u32(), {self.ptr += 4}).0
    }

    #[inline]
    pub fn read_n(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.read_u8()).collect()
    }

    pub fn read_name(&mut self) -> String {
        if self.peek_u8() & 0xC0 > 0 {
            let mut copy = *self;
            copy.ptr = (self.read_u16() & !0xC000) as usize;
            copy.read_name()
        } else {
            let mut cname = Vec::new();
            loop {
                let len = self.read_u8();
                if len == 0 {break}
                cname.push((0..len).map(|_| std::str::from_utf8(&[self.read_u8()]).expect("Invalid ASCII character.").to_string()).collect::<Vec<String>>().concat());
            }
            cname.join(".")
        }
    }
}
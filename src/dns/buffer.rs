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
        let mut cname: Vec<String> = Vec::new();
        loop {
            let len = self.peek_u8();
            if len == 0 { self.ptr += 1; return cname.join(".") }
            if len & 0xC0u8 == 0xC0u8 {
                let mut copy = *self;
                copy.ptr = (self.read_u16() & !0xC000u16) as usize;
                cname.push(copy.read_name());
                return cname.join(".")
            } else {
                cname.push((0..self.read_u8()).map(|_| {
                    std::str::from_utf8(&[self.read_u8()]).expect("Invalid ASCII character.").to_string()
                }).collect::<Vec<String>>().concat());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::DnsPacketBuffer;

    #[test]
    fn test_read_u16() {
        let mut buffer = DnsPacketBuffer::new(&[0xc0, 0x0c, 0x11]); 
        assert_eq!(buffer.read_u16(), 0xc00cu16);
        assert_eq!(buffer.peek_u8(), 0x11u8);
    }

    #[test]
    fn test_read_name() {
        let mut buffer = DnsPacketBuffer::new(&[0x3, 'a' as u8, 'b' as u8, 'c' as u8, 0x0]);
        assert_eq!(buffer.read_name(), "abc".to_string())
    }
}
use std::io;

/*
/// Bitwise stream reader
pub struct BitStreamReader<'a> {
    buffer: [u8; 1],
    offset: u8,
    reader: &'a mut dyn io::Read,
}

impl<'a> BitStreamReader<'a> {
    /// Create a new BitStreamReader that reads bitwise from a given reader
    pub fn new(reader: &'a mut dyn io::Read) -> BitStreamReader {
        BitStreamReader {
            buffer: [0u8],
            reader: reader,
            offset: 8,
        }
    }

    /// Read nbit bits
    pub fn read(&mut self, mut nbits: u8) -> Result<u64, io::Error> {
        if nbits > 64 {
            return Err(io::Error::new(io::ErrorKind::Other, "can not read more than 64 bits at once"));
        }
        let mut data = 0u64;
        while nbits > 0 {
            if self.offset == 8 {
                self.reader.read_exact(&mut self.buffer)?;
                self.offset = 0;
            }
            let bits = std::cmp::min(8 - self.offset, nbits);
            data <<= bits;
            data |= ((self.buffer[0] << self.offset) >> (8 - bits)) as u64;
            self.offset += bits;
            nbits -= bits;
        }
        Ok(data)
    }
}
*/

/// Bitwise stream writer
pub struct BitStreamWriter<'a> {
    buffer: [u8; 1],
    offset: u8,
    writer: &'a mut dyn io::Write,
}

impl<'a> BitStreamWriter<'a> {
    /// Create a new BitStreamWriter that writes bitwise to a given writer
    pub fn new(writer: &'a mut dyn io::Write) -> BitStreamWriter {
        BitStreamWriter {
            buffer: [0u8],
            writer: writer,
            offset: 0,
        }
    }

    /// Write nbits bits from data
    pub fn write(&mut self, data: u64, mut nbits: u8) -> Result<usize, io::Error> {
        if nbits > 64 {
            return Err(io::Error::new(io::ErrorKind::Other, "can not write more than 64 bits at once"));
        }
        let mut wrote = 0;
        while nbits > 0 {
            let bits = std::cmp::min(8 - self.offset, nbits);
            self.buffer[0] |= ((data << (64 - nbits)) >> (64 - 8 + self.offset)) as u8;
            self.offset += bits;
            nbits -= bits;
            if self.offset == 8 {
                wrote += self.flush()?;
            }
        }
        Ok(wrote)
    }

    /// flush bits not yet written
    pub fn flush(&mut self) -> Result<usize, io::Error> {
        if self.offset > 0 {
            self.writer.write_all(&self.buffer)?;
            self.buffer[0] = 0u8;
            self.offset = 0;
            Ok(1)
        } else {
            Ok(0)
        }
    }
}
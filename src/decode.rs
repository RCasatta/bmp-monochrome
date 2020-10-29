use crate::bit::BitStreamReader;
use crate::{check_size, Bmp, BmpError, BmpHeader, B, HEADER_SIZE, M};
use std::convert::TryFrom;
use std::io::{Cursor, Read};

impl Bmp {
    /// Read the monochrome bitmap from a Read type, such a File
    /// note that File read are not buffered and may be slow, see [Read](std::io::Read) Trait
    pub fn read<T: Read>(mut from: T) -> Result<Self, BmpError> {
        let mut header_bytes = [0u8; HEADER_SIZE as usize];
        from.read_exact(&mut header_bytes)?;
        let header = BmpHeader::read(Cursor::new(&mut header_bytes.to_vec()))?;
        let width = header.width;
        let height = header.height;
        let padding = header.padding() as u8;
        let mut reader = BitStreamReader::new(&mut from);
        let mut rows = Vec::with_capacity(height as usize);
        let mut row = Vec::with_capacity(width as usize);
        for _ in 0..height as usize {
            for _ in 0..width as usize {
                if reader.read(1)? == 1 {
                    row.push(true);
                } else {
                    row.push(false);
                }
            }
            reader.read((8 - (width % 8) as u8) % 8)?; // finish reading the full byte
            reader.read(padding * 8)?; // read the padding such that every row is multiple of 4 bytes
            rows.push(row.clone());
            row.clear();
        }
        rows.reverse();

        Ok(Bmp { rows })
    }
}

impl BmpHeader {
    /// read the BmpHeader from read Trait `T`
    /// returns `BmpError::Size` for error related to the declared bmp size, see `check_size`
    /// and `BmpError::Header` for any other error
    pub fn read<T: Read>(mut from: T) -> Result<Self, BmpError> {
        let b = ReadLE::read_u8(&mut from)?;
        let m = ReadLE::read_u8(&mut from)?;
        let _total_size = ReadLE::read_u32(&mut from)?;
        let _creator1 = ReadLE::read_u16(&mut from)?;
        let _creator2 = ReadLE::read_u16(&mut from)?;
        let pixel_offset = ReadLE::read_u32(&mut from)?;
        let dib_header = ReadLE::read_u32(&mut from)?;
        let width = ReadLE::read_u32(&mut from)?;
        let height = ReadLE::read_u32(&mut from)?;
        let planes = ReadLE::read_u16(&mut from)?;
        let bits_per_pixel = ReadLE::read_u16(&mut from)?;
        let compression = ReadLE::read_u32(&mut from)?;
        let _data_size = ReadLE::read_u32(&mut from)?;
        let _hres = ReadLE::read_u32(&mut from)?;
        let _vres = ReadLE::read_u32(&mut from)?;
        let num_colors = ReadLE::read_u32(&mut from)?;
        let _num_imp_colors = ReadLE::read_u32(&mut from)?;
        let _background_color = ReadLE::read_u32(&mut from)?;
        let _foreground_color = ReadLE::read_u32(&mut from)?;

        if b != B
            || m != M
            || pixel_offset != HEADER_SIZE
            || dib_header != 40u32
            || planes != 1u16
            || bits_per_pixel != 1u16
            || compression != 0u32
            || num_colors != 2u32
        {
            return Err(BmpError::Header);
        }

        let width = u16::try_from(width)?;
        let height = u16::try_from(height)?;
        check_size(width, height)?;

        Ok(BmpHeader { height, width })
    }
}

impl<R: Read> ReadLE for R {
    fn read_u32(&mut self) -> Result<u32, BmpError> {
        let mut buffer = [0u8; 4];
        self.read_exact(&mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_u16(&mut self) -> Result<u16, BmpError> {
        let mut buffer = [0u8; 2];
        self.read_exact(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    fn read_u8(&mut self) -> Result<u8, BmpError> {
        let mut buffer = [0u8];
        self.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }
}

trait ReadLE {
    /// Read a 32-bit uint
    fn read_u32(&mut self) -> Result<u32, BmpError>;
    /// Read a 16-bit uint
    fn read_u16(&mut self) -> Result<u16, BmpError>;
    /// Read a 8-bit uint
    fn read_u8(&mut self) -> Result<u8, BmpError>;
}

#[cfg(test)]
mod test {
    use crate::decode::ReadLE;
    use crate::BmpHeader;
    use std::fs::File;
    use std::io::Cursor;

    #[test]
    fn test_read() {
        let mut cursor = Cursor::new(vec![0u8, 1, 1, 0, 1, 0, 0, 0]);
        assert_eq!(0, ReadLE::read_u8(&mut cursor).unwrap());
        assert_eq!(1, ReadLE::read_u8(&mut cursor).unwrap());
        assert_eq!(1, ReadLE::read_u16(&mut cursor).unwrap());
        assert_eq!(1, ReadLE::read_u32(&mut cursor).unwrap());
    }

    #[test]
    fn test_header() {
        let file = File::open("test_bmp/monochrome_image.bmp").unwrap();
        let bmp_header = BmpHeader::read(file).unwrap();
        assert_eq!(18, bmp_header.width);
        assert_eq!(18, bmp_header.height);

        let file = File::open("test_bmp/test1.bmp").unwrap();
        let bmp_header = BmpHeader::read(file).unwrap();
        assert_eq!(2, bmp_header.width);
        assert_eq!(2, bmp_header.height);
    }
}

use crate::bit::BitStreamWriter;
use crate::{Bmp, BmpError, BmpHeader, B, HEADER_SIZE, M};
use std::io::Write;

impl Bmp {
    /// Write the monochrome bitmap to a Write type, such a File
    pub fn write<T: Write>(&self, mut to: T) -> Result<(), BmpError> {
        let height = self.height();
        let width = self.width();

        let header = BmpHeader {
            height,
            width,
            bg_is_zero: false,
        };
        let padding = header.padding() as u8;

        header.write(&mut to)?;

        let mut writer = BitStreamWriter::new(&mut to);

        for i in (0..self.height()).rev() {
            for j in 0..self.width() {
                if self.get(i, j) {
                    writer.write(1, 1)?;
                } else {
                    writer.write(0, 1)?;
                }
            }
            writer.write(0, (8 - (width % 8) as u8) % 8)?;
            writer.write(0, padding * 8)?;
        }
        writer.flush()?;

        Ok(())
    }
}

impl BmpHeader {
    pub fn write<T: Write>(&self, to: &mut T) -> Result<(), BmpError> {
        let bytes_per_row = self.bytes_per_row();
        let padding = self.padding();
        let data_size = (bytes_per_row + padding) * (self.height as u32);
        let total_size = HEADER_SIZE + data_size;

        to.write_all(&[B, M])?;
        to.write_all(&total_size.to_le_bytes())?; // size of the bmp
        to.write_all(&0u16.to_le_bytes())?; // creator1
        to.write_all(&0u16.to_le_bytes())?; // creator2
        to.write_all(&HEADER_SIZE.to_le_bytes())?; // pixel offset
        to.write_all(&40u32.to_le_bytes())?; // dib header size
        to.write_all(&(self.width as u32).to_le_bytes())?; // width
        to.write_all(&(self.height as u32).to_le_bytes())?; // height
        to.write_all(&1u16.to_le_bytes())?; // planes
        to.write_all(&1u16.to_le_bytes())?; // bitsperpixel
        to.write_all(&0u32.to_le_bytes())?; // no compression
        to.write_all(&data_size.to_le_bytes())?; // size of the raw bitmap data with padding
        to.write_all(&512u32.to_le_bytes())?; // hres
        to.write_all(&512u32.to_le_bytes())?; // vres
        to.write_all(&2u32.to_le_bytes())?; // num_colors
        to.write_all(&2u32.to_le_bytes())?; // num_imp_colors

        if self.bg_is_zero {
            to.write_all(&0x00_00_00_00u32.to_le_bytes())?; // color_pallet 0
            to.write_all(&0x00_FF_FF_FFu32.to_le_bytes())?; // color_pallet 1
        } else {
            to.write_all(&0x00_FF_FF_FFu32.to_le_bytes())?; // color_pallet 0
            to.write_all(&0x00_00_00_00u32.to_le_bytes())?; // color_pallet 1
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::Bmp;
    use std::io::Cursor;

    #[test]
    fn test_width8() {
        // from issue https://github.com/RCasatta/bmp-monochrome/issues/2

        let data: Vec<bool> = vec![
            true, true, true, true, true, true, true, true, true, false, false, false, false,
            false, false, true, true, false, false, false, false, false, false, true, true, true,
            true, true, true, true, true, true,
        ];
        let vec: Vec<Vec<bool>> = data.chunks(8).map(|c| c.to_vec()).collect();

        let bmp_created = Bmp::new(vec).unwrap();
        let mut buffer = Cursor::new(vec![]);
        bmp_created.write(&mut buffer).unwrap();
        buffer.set_position(0);
        let bmp = Bmp::read(buffer).unwrap();
        assert_eq!(bmp_created, bmp);
    }
}

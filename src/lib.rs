mod bit;

use std::error::Error;

const B: u8 = 66;
const M: u8 = 77;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataMatrix {
    data: Vec<bool>,
    width: usize,
}

impl DataMatrix {
    fn height(&self) -> usize {
        self.data.len() / self.width
    }

    fn get(&self, i: usize, j: usize) -> Option<&bool> {
        let h = self.height() - i - 1;
        self.data.get(h * self.width + j)
    }

    /// multiply by `mul` every pixel
    pub fn mul(&self, mul: usize) -> DataMatrix {
        let mut data = vec![];

        for i in 0..self.height() {
            let mut row = vec![];
            for j in 0..self.width {
                for _ in 0..mul {
                    row.push(self.get(i, j).unwrap());
                }
            }
            for _ in 0..mul {
                data.extend(row.clone());
            }
        }

        let width = self.width * mul;
        DataMatrix { data, width }
    }

    /// add `white_space` pixels around
    pub fn add_whitespace(&self, white_space: usize) -> DataMatrix {
        let width = self.width + white_space * 2;
        let mut data = vec![];
        for _ in 0..white_space {
            data.extend(vec![false; width]);
        }
        for vec in self.data.chunks(self.width) {
            for _ in 0..white_space {
                data.push(false);
            }
            data.extend(vec);
            for _ in 0..white_space {
                data.push(false);
            }
        }
        for _ in 0..white_space {
            data.extend(vec![false; width]);
        }

        DataMatrix { data, width }
    }

    /// Returns a monocromatic bitmap
    pub fn bmp(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let matrix = self.clone();
        let width = matrix.width as u32;
        let height = matrix.height() as u32;

        let mut bmp_data = vec![];
        let header = BmpHeader { height, width };
        let padding = header.padding() as u8;
        bmp_data.extend(header.write());

        let mut data = Vec::new();
        let mut writer = bit::BitStreamWriter::new(&mut data);

        for i in 0..height as usize {
            for j in 0..width as usize {
                if *matrix.get(i, j).unwrap() {
                    writer.write(1, 1)?;
                } else {
                    writer.write(0, 1)?;
                }
            }
            writer.write(0, 8 - (width % 8) as u8)?; // 0
            writer.write(0, padding * 8)?; // 0
        }
        writer.flush().unwrap();
        bmp_data.extend(data);

        Ok(bmp_data)
    }
}

struct BmpHeader {
    height: u32,
    width: u32,
}

impl BmpHeader {
    /*pub fn from_bytes(_bytes: Vec<u8>) {
        unimplemented!();
    }*/

    pub fn write(&self) -> Vec<u8> {
        let color_pallet_size = 2 * 4; // 2 colors each 4 bytes
        let header_size = 2 + 12 + 40 + color_pallet_size;
        let bytes_per_row = self.bytes_per_row();
        let padding = self.padding();
        let data_size = (bytes_per_row + padding) * (self.height as u32);
        let total_size = header_size + data_size;
        let mut output = vec![];

        // https://en.wikipedia.org/wiki/BMP_file_format
        output.push(B);
        output.push(M);
        output.extend(&total_size.to_le_bytes()); // size of the bmp
        output.extend(&0u16.to_le_bytes()); // creator1
        output.extend(&0u16.to_le_bytes()); // creator2
        output.extend(&header_size.to_le_bytes()); // pixel offset
        output.extend(&40u32.to_le_bytes()); // dib header size
        output.extend(&(self.width as u32).to_le_bytes()); // width
        output.extend(&(self.height as u32).to_le_bytes()); // height
        output.extend(&1u16.to_le_bytes()); // planes
        output.extend(&1u16.to_le_bytes()); // bitsperpixel
        output.extend(&0u32.to_le_bytes()); // no compression
        output.extend(&data_size.to_le_bytes()); // size of the raw bitmap data with padding
        output.extend(&2835u32.to_le_bytes()); // hres
        output.extend(&2835u32.to_le_bytes()); // vres
        output.extend(&2u32.to_le_bytes()); // num_colors
        output.extend(&2u32.to_le_bytes()); // num_imp_colors

        // color_pallet
        output.extend(&0x00_FF_FF_FFu32.to_le_bytes());
        output.extend(&0x00_00_00_00u32.to_le_bytes());

        output
    }

    /// return bytes needed for `width` bits
    fn bytes_per_row(&self) -> u32 {
        (self.width + 7) / 8
    }

    /// return the padding
    fn padding(&self) -> u32 {
        (4 - self.bytes_per_row() % 4) % 4
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_padding() {
        let mut header = BmpHeader {
            height: 0,
            width: 0,
        };
        assert_eq!(header.padding(), 0);

        header.width = 1;
        assert_eq!(header.padding(), 3);

        header.width = 9;
        assert_eq!(header.padding(), 2);

        header.width = 17;
        assert_eq!(header.padding(), 1);

        header.width = 25;
        assert_eq!(header.padding(), 0);
    }

    #[test]
    fn test_bytes_per_row() {
        let mut header = BmpHeader {
            height: 0,
            width: 0,
        };
        assert_eq!(header.bytes_per_row(), 0);

        header.width= 1;
        assert_eq!(header.bytes_per_row(), 1);

        header.width= 8;
        assert_eq!(header.bytes_per_row(), 1);

        header.width= 9;
        assert_eq!(header.bytes_per_row(), 2);
    }

    #[test]
    fn test_mul() {
        let data = DataMatrix {
            data: vec![false, true, false, true],
            width: 2,
        };

        let data_bigger = DataMatrix {
            data: vec![
                false, false, true, true, false, false, true, true, false, false, true, true,
                false, false, true, true,
            ],
            width: 4,
        };

        assert_eq!(data.mul(2), data_bigger);
    }

    #[test]
    fn test_add() {
        let data = DataMatrix {
            data: vec![false],
            width: 1,
        };

        let data_bigger = DataMatrix {
            data: vec![false; 25],
            width: 5,
        };

        assert_eq!(data.add_whitespace(2), data_bigger);
    }

    #[test]
    fn test_bmp() {
        let data_test1 = DataMatrix {
            data: vec![false, true, true, false],
            width: 2,
        };
        let bmp_test1 = data_test1.bmp().unwrap();
        let bytes_test1 = include_bytes!("../test_bmp/test1.bmp").to_vec();
        assert_eq!(bmp_test1, bytes_test1);

        let bmp_test2 = data_test1.mul(3).add_whitespace(12).bmp().unwrap();
        let bytes_test2 = include_bytes!("../test_bmp/test2.bmp").to_vec();
        assert_eq!(bmp_test2, bytes_test2);
    }

    /*
    #[test]
    fn test_monochrome_image() {
        // taken from https://github.com/pertbanking/bitmap-monochrome/blob/master/monochrome_image.bmp
        let data = DataMatrix {
            data: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
                       0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0,
                       0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0,
                       0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0,
                       0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0,
                       0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,
                       0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0,
                       0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0,
                       0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0,
                       0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0,
                       0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0,
                       0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0,
                       0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0,
                       0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
                       0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                       0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].iter().map(|e| *e==0).collect(),
            width: 18,
        };
        let bmp_test = data.bmp().unwrap();
        let mut file = File::create("test_bmp/monochrome_image_lib.bmp").unwrap();
        file.write_all(&bmp_test).unwrap();
        let bytes_test = include_bytes!("../test_bmp/monochrome_image.bmp").to_vec();
        let bytes_padding = include_bytes!("../test_bmp/monochrome_image_padding.bmp").to_vec();
        let bytes_final: Vec<u8> = bytes_test.iter().zip(bytes_padding).map(|e| e.0 & e.1).collect();
        assert_eq!(bmp_test, bytes_final);
    }
    */
}

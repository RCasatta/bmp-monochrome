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
        let width = matrix.width;
        let height = matrix.height();

        let color_pallet_size = 2 * 4; // 2 colors each 4 bytes
        let header_size = 2 + 12 + 40 + color_pallet_size;
        let bytes_per_row = bytes_per_row(width as u32);
        let padding = padding(bytes_per_row);
        let data_size = (bytes_per_row + padding) * (height as u32);
        let total_size = header_size + data_size;
        let mut bmp_data = vec![];

        // https://en.wikipedia.org/wiki/BMP_file_format
        bmp_data.push(B);
        bmp_data.push(M);
        bmp_data.extend(&total_size.to_le_bytes()); // size of the bmp
        bmp_data.extend(&0u16.to_le_bytes()); // creator1
        bmp_data.extend(&0u16.to_le_bytes()); // creator2
        bmp_data.extend(&header_size.to_le_bytes()); // pixel offset
        bmp_data.extend(&40u32.to_le_bytes()); // dib header size
        bmp_data.extend(&(width as u32).to_le_bytes()); // width
        bmp_data.extend(&(height as u32).to_le_bytes()); // height
        bmp_data.extend(&1u16.to_le_bytes()); // planes
        bmp_data.extend(&1u16.to_le_bytes()); // bitsperpixel
        bmp_data.extend(&0u32.to_le_bytes()); // no compression
        bmp_data.extend(&data_size.to_le_bytes()); // size of the raw bitmap data with padding
        bmp_data.extend(&2835u32.to_le_bytes()); // hres
        bmp_data.extend(&2835u32.to_le_bytes()); // vres
        bmp_data.extend(&2u32.to_le_bytes()); // num_colors
        bmp_data.extend(&2u32.to_le_bytes()); // num_imp_colors

        // color_pallet
        bmp_data.extend(&0x00_FF_FF_FFu32.to_le_bytes());
        bmp_data.extend(&0x00_00_00_00u32.to_le_bytes());

        let mut data = Vec::new();
        let mut writer = bit::BitStreamWriter::new(&mut data);

        for i in 0..height {
            for j in 0..width {
                if *matrix.get(i, j).unwrap() {
                    writer.write(1, 1)?;
                } else {
                    writer.write(0, 1)?;
                }
            }
            writer.write(0, 8 - (width % 8) as u8)?; // 0
            writer.write(0, padding as u8 * 8)?; // 0
        }
        writer.flush().unwrap();
        bmp_data.extend(data);

        Ok(bmp_data)
    }
}

/// return bytes needed for `width` bits
fn bytes_per_row(width: u32) -> u32 {
    (width + 7) / 8
}

/// return the padding needed for n
fn padding(n: u32) -> u32 {
    (4 - n % 4) % 4
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_padding() {
        assert_eq!(padding(0), 0);
        assert_eq!(padding(1), 3);
        assert_eq!(padding(2), 2);
        assert_eq!(padding(3), 1);
        assert_eq!(padding(4), 0);
    }

    #[test]
    fn test_bytes_per_row() {
        assert_eq!(bytes_per_row(0), 0);
        assert_eq!(bytes_per_row(1), 1);
        assert_eq!(bytes_per_row(3), 1);
        assert_eq!(bytes_per_row(8), 1);
        assert_eq!(bytes_per_row(9), 2);
        assert_eq!(bytes_per_row(64), 8);
        assert_eq!(bytes_per_row(65), 9);
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

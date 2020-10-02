//! # BMP monochrome
//!
//! This library encode and decode monochromatic bitmap with no extra dependencies.
//! Especially useful to encode QR-codes
//!

#![deny(missing_docs)]

use std::io::Error;

mod bit;
mod decode;
mod encode;

const B: u8 = 66;
const M: u8 = 77;
const COLOR_PALLET_SIZE: u32 = 2 * 4; // 2 colors each 4 bytes
const HEADER_SIZE: u32 = 2 + 12 + 40 + COLOR_PALLET_SIZE;

/// The `Bmp` struct contains the data as a vector of boolean, each representing a pixel.
/// In `data` the first element is the upper-left pixel, then proceed in the row.
/// Last element of `data` is the lower-right pixel.
/// Note in the serialized format the first element is the lower-left pixel
/// see https://en.wikipedia.org/wiki/BMP_file_format
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Bmp {
    data: Vec<bool>,
    width: usize,
}

/// Internal error struct
#[derive(Debug)]
pub enum BmpError {
    /// Generic
    Generic,
    /// Relative to the content
    Content,
    /// Relative to the header
    Header,
    /// Relative to the data
    Data,
}

#[derive(Debug)]
struct BmpHeader {
    height: u32,
    width: u32,
}

impl Bmp {
    /// Creates a new DataMatrix, failing if `data` is empty or its length not a multiple of `width`
    pub fn new(data: Vec<bool>, width: usize) -> Result<Bmp, BmpError> {
        if data.is_empty() || width == 0 || data.len() % width != 0 {
            Err(BmpError::Data)
        } else {
            Ok(Bmp { data, width })
        }
    }

    fn height(&self) -> usize {
        self.data.len() / self.width
    }

    /// could panic if (i * self.height() + j) >= self.data.len()
    fn get(&self, i: usize, j: usize) -> bool {
        let h = self.height() - i - 1;
        self.data[h * self.width + j]
    }

    /// multiply by `mul` every pixel
    pub fn mul(&self, mul: usize) -> Bmp {
        let mut data = vec![];

        for i in 0..self.height() {
            let mut row = vec![];
            for j in 0..self.width {
                for _ in 0..mul {
                    row.push(self.get(i, j));
                }
            }
            for _ in 0..mul {
                data.extend(row.clone());
            }
        }

        let width = self.width * mul;
        Bmp { data, width }
    }

    /// add `white_space` pixels around
    pub fn add_whitespace(&self, white_space: usize) -> Bmp {
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

        Bmp { data, width }
    }
}

impl From<std::io::Error> for BmpError {
    fn from(_: Error) -> Self {
        BmpError::Generic
    }
}

impl BmpHeader {
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
    use rand::Rng;
    use std::fs::File;
    use std::io::Cursor;

    #[test]
    fn test_data_matrix() {
        assert!(Bmp::new(vec![], 1).is_err());
        assert!(Bmp::new(vec![true], 0).is_err());
        assert!(Bmp::new(vec![true], 1).is_ok());
        assert!(Bmp::new(vec![true], 2).is_err());
        assert!(Bmp::new(vec![true, false], 2).is_ok());
        assert!(Bmp::new(vec![true, false], 1).is_ok());
        assert!(Bmp::new(vec![true, false, true], 1).is_ok());
        assert!(Bmp::new(vec![true, false, true], 2).is_err());
    }

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

        header.width = 1;
        assert_eq!(header.bytes_per_row(), 1);

        header.width = 8;
        assert_eq!(header.bytes_per_row(), 1);

        header.width = 9;
        assert_eq!(header.bytes_per_row(), 2);
    }

    #[test]
    fn test_mul() {
        let data = Bmp {
            data: vec![false, true, false, true],
            width: 2,
        };

        let data_bigger = Bmp {
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
        let data = Bmp {
            data: vec![false],
            width: 1,
        };

        let data_bigger = Bmp {
            data: vec![false; 25],
            width: 5,
        };

        assert_eq!(data.add_whitespace(2), data_bigger);
    }

    #[test]
    fn test_bmp() {
        let data_test1 = Bmp {
            data: vec![false, true, true, false],
            width: 2,
        };
        let bytes_test1 = Bmp::read(&mut File::open("test_bmp/test1.bmp").unwrap()).unwrap();
        assert_eq!(data_test1, bytes_test1);

        let bmp_test2 = data_test1.mul(3).add_whitespace(12);
        let bytes_test2 = Bmp::read(&mut File::open("test_bmp/test2.bmp").unwrap()).unwrap();
        assert_eq!(bmp_test2, bytes_test2);
    }

    #[test]
    fn test_monochrome_image() {
        // taken from https://github.com/pertbanking/bitmap-monochrome/blob/master/monochrome_image.bmp
        let expected = Bmp {
            data: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
                0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0,
                0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1,
                1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0,
                1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0,
                1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1,
                0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0,
                0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .iter()
            .map(|e| *e != 0)
            .collect(),
            width: 18,
        };
        let bmp = Bmp::read(File::open("test_bmp/monochrome_image.bmp").unwrap()).unwrap();
        assert_eq!(expected, bmp);
    }

    #[test]
    fn test_rtt() {
        let mut rng = rand::thread_rng();
        let width = rng.gen_range(1, 20);
        let height = rng.gen_range(1, 20);
        let data: Vec<bool> = (0..width * height).map(|_| rng.gen()).collect();
        let expected = Bmp::new(data, width).unwrap();
        let mut cursor = Cursor::new(vec![]);
        expected.write(&mut cursor).unwrap();
        cursor.set_position(0);
        let bmp = Bmp::read(&mut cursor).unwrap();
        assert_eq!(expected, bmp);
    }
}

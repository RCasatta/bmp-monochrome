//! # BMP monochrome
//!
//! This library encode and decode monochromatic bitmap with no extra dependencies.
//! Especially useful to encode QR-codes
//!

#![deny(missing_docs)]

use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error;
use std::num::TryFromIntError;

mod bit;
mod decode;
mod encode;

#[cfg(feature = "fuzz")]
pub mod fuzz;

const B: u8 = 66;
const M: u8 = 77;
const COLOR_PALLET_SIZE: u32 = 2 * 4; // 2 colors each 4 bytes
const HEADER_SIZE: u32 = 2 + 12 + 40 + COLOR_PALLET_SIZE;

/// The `Bmp` struct contains the data as a vector of boolean, each representing a pixel.
/// In `data` the first element is the upper-left pixel, then proceed in the row.
/// Last element of `data` is the lower-right pixel.
/// Note in the serialized format the first element is the lower-left pixel
/// see [BMP file format](https://en.wikipedia.org/wiki/BMP_file_format)
#[derive(PartialEq, Eq, Clone)]
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
    /// Relative to the size
    Size(u32, u32),
}

impl Display for BmpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for Bmp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.data.len() < 50 {
            write!(f, "Bmp data={:?} width={:?}", self.data, self.width)
        } else {
            write!(
                f,
                "Bmp data.len()={} width={:?}",
                self.data.len(),
                self.width
            )
        }
    }
}

impl std::error::Error for BmpError {}

impl From<std::num::TryFromIntError> for BmpError {
    fn from(_: TryFromIntError) -> Self {
        BmpError::Generic
    }
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
            let height = data.len() / width;
            check_size(u32::try_from(width)?, u32::try_from(height)?)?;
            Ok(Bmp { data, width })
        }
    }

    /// return the Bmp height in pixel
    pub fn height(&self) -> usize {
        self.data.len() / self.width
    }

    /// return the Bmp width in pixel
    pub fn width(&self) -> usize {
        self.width
    }

    /// return the pixel situated at (i,j), where (0,0) is the upper-left corner
    /// could panic if (i * self.height() + j) >= self.data.len()
    pub fn pixel(&self, i: usize, j: usize) -> bool {
        let h = self.height() - i - 1;
        self.data[h * self.width + j]
    }

    /// return the pixel situated at (x,y) where (0,0) is the lower-left corner
    /// could panic if (i * self.height() + j) >= self.data.len()
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.data[x * self.width + y]
    }

    /// return a new Bmp where every pixel is multiplied by `mul`
    pub fn mul(&self, mul: usize) -> Result<Bmp, BmpError> {
        let new_width = self
            .width
            .checked_mul(mul)
            .ok_or_else(|| BmpError::Generic)?;
        let new_height = self
            .height()
            .checked_mul(mul)
            .ok_or_else(|| BmpError::Generic)?;
        check_size(u32::try_from(new_width)?, u32::try_from(new_height)?)?;
        let total = new_width * new_height;
        let mut data = Vec::with_capacity(total);

        for i in 0..self.height() {
            let mut row = Vec::with_capacity(new_width);
            for j in 0..self.width {
                for _ in 0..mul {
                    row.push(self.get(i, j));
                }
            }
            for _ in 0..mul {
                data.extend(row.clone());
            }
        }

        let width = new_width;
        Ok(Bmp { data, width })
    }

    /// return a new Bmp where every square is divided by `div`
    /// if all the square is not of the same color it errors
    pub fn div(&self, div: usize) -> Result<Bmp, BmpError> {
        if div <= 1 {
            return Err(BmpError::Generic);
        }
        let new_width = self.width / div;
        if new_width == 0 {
            return Err(BmpError::Generic);
        }
        let mut new_data = vec![];

        for (i, chunk) in self.data.chunks(div).enumerate() {
            let row = i / new_width;
            if chunk.iter().all(|e| chunk[0] == *e) {
                if row % div == 0 {
                    new_data.push(chunk[0]);
                }
            } else {
                return Err(BmpError::Generic);
            }
        }
        Bmp::new(new_data, new_width)
    }

    fn div_with_greater_possible(&self, greater_start: usize) -> Bmp {
        for i in (2..greater_start).rev() {
            if let Ok(bmp) = self.div(i) {
                return bmp;
            }
        }
        self.clone()
    }

    /// `normalize` removes the white border if any, and reduce the module pixel size to 1
    /// (the module must be smaller than 10x10 pixel)
    pub fn normalize(&self) -> Bmp {
        self.remove_white_border().div_with_greater_possible(10)
    }

    /// return a new Bmp with `border_size` pixels around
    pub fn add_white_border(&self, border_size: usize) -> Result<Bmp, BmpError> {
        let double_border = border_size
            .checked_mul(2)
            .ok_or_else(|| BmpError::Generic)?;
        let width = self
            .width
            .checked_add(double_border)
            .ok_or_else(|| BmpError::Generic)?;
        let height = self
            .height()
            .checked_add(double_border)
            .ok_or_else(|| BmpError::Generic)?;
        check_size(u32::try_from(width)?, u32::try_from(height)?)?;
        let mut data = Vec::with_capacity(width * height);
        data.extend(vec![false; width * border_size]);
        for vec in self.data.chunks(self.width) {
            data.extend(vec![false; border_size]);
            data.extend(vec);
            data.extend(vec![false; border_size]);
        }
        data.extend(vec![false; width * border_size]);

        Ok(Bmp { data, width })
    }

    /// remove all the white border, if any
    pub fn remove_white_border(&self) -> Bmp {
        let mut cur = self.clone();
        loop {
            match cur.remove_one_white_border() {
                Ok(bmp) => cur = bmp,
                Err(_) => return cur,
            }
        }
    }

    fn remove_one_white_border(&self) -> Result<Bmp, BmpError> {
        let border_is_white = self
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_border(*i))
            .all(|(_, e)| !*e);
        if self.width > 2 && border_is_white && self.height() > 2 {
            let data: Vec<bool> = self
                .data
                .iter()
                .enumerate()
                .filter_map(|(i, e)| if self.is_border(i) { None } else { Some(*e) })
                .collect();
            Ok(Bmp::new(data, self.width - 2).unwrap())
        } else {
            Err(BmpError::Generic)
        }
    }

    fn is_border(&self, i: usize) -> bool {
        let max_h = self.height();
        let max_w = self.width;
        let h = i / max_w;
        let w = i % max_w;

        h == 0 || h == max_h - 1 || w == 0 || w == max_w - 1
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

/// arbitrary limit width * height < 100 million
/// height and width must be > 0
fn check_size(width: u32, height: u32) -> Result<(), BmpError> {
    let width_height = width
        .checked_mul(height)
        .ok_or_else(|| BmpError::Size(width, height))?;
    if width_height <= 1_000_000 && width > 0 && height > 0 {
        Ok(())
    } else {
        Err(BmpError::Size(width, height))
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

        assert_eq!(data.mul(2).unwrap(), data_bigger);

        let data = Bmp {
            data: vec![false, false, false, true],
            width: 2,
        };

        let data_bigger = Bmp {
            data: vec![
                false, false, false, false, false, false, false, false, false, false, true, true,
                false, false, true, true,
            ],
            width: 4,
        };

        assert_eq!(data.mul(2).unwrap(), data_bigger);
    }

    #[test]
    fn test_div() {
        let data = Bmp {
            data: vec![false, false, true, true, false, false, true, true],
            width: 4,
        };
        let expected = Bmp {
            data: vec![false, true],
            width: 2,
        };
        assert_eq!(expected, data.div(2).unwrap());
    }

    #[test]
    fn test_mul_div() {
        let expected = random_bmp();
        let mul = expected.mul(3).unwrap();
        let div = mul.div(3).unwrap();
        assert_eq!(expected, div);
    }

    #[test]
    fn test_add_white_border() {
        let data = Bmp {
            data: vec![false],
            width: 1,
        };

        let data_bigger = Bmp {
            data: vec![false; 25],
            width: 5,
        };

        assert_eq!(data.add_white_border(2).unwrap(), data_bigger);
    }

    #[test]
    fn test_bmp() {
        let data_test1 = Bmp {
            data: vec![false, true, true, false],
            width: 2,
        };
        let bytes_test1 = Bmp::read(&mut File::open("test_bmp/test1.bmp").unwrap()).unwrap();
        assert_eq!(data_test1, bytes_test1);

        let bmp_test2 = data_test1.mul(3).unwrap().add_white_border(12).unwrap();
        bmp_test2
            .write(File::create("test_bmp/test2.bmp").unwrap())
            .unwrap();
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
    fn test_rt() {
        let expected = random_bmp();
        let mut cursor = Cursor::new(vec![]);
        expected.write(&mut cursor).unwrap();
        cursor.set_position(0);
        let bmp = Bmp::read(&mut cursor).unwrap();
        assert_eq!(expected, bmp);
    }

    #[test]
    fn test_get_and_pixel() {
        let file = File::open("test_bmp/test1.bmp").unwrap();
        let bmp = Bmp::read(file).unwrap();
        assert!(!bmp.get(0, 0), "lower-left is not dark");
        assert!(bmp.pixel(0, 0), "upper-left is not white");
    }

    #[test]
    fn test_is_border() {
        let bmp = Bmp {
            data: vec![false; 9],
            width: 3,
        };
        for i in 0..9 {
            assert_eq!(bmp.is_border(i), i != 4);
        }

        let bmp = Bmp {
            data: vec![false; 16],
            width: 4,
        };
        for i in 0..16 {
            assert_eq!(bmp.is_border(i), ![5, 6, 9, 10].contains(&i));
        }
    }

    #[test]
    fn test_remove_white_border() {
        let bmp = Bmp {
            data: vec![false; 25],
            width: 5,
        };
        let expected = Bmp {
            data: vec![false; 1],
            width: 1,
        };
        assert_eq!(expected, bmp.remove_white_border());
    }

    #[test]
    fn test_div_with_greater_possible() {
        let bmp = random_bmp();
        let mul = bmp.mul(4).unwrap();
        let div = mul.div_with_greater_possible(10);
        assert_eq!(div, bmp);
    }

    #[test]
    fn test_normalize() {
        let bmp = Bmp::read(File::open("test_bmp/qr_not_normalized.bmp").unwrap()).unwrap();
        let bmp_normalized = Bmp::read(File::open("test_bmp/qr_normalized.bmp").unwrap()).unwrap();
        assert_eq!(bmp.normalize(), bmp_normalized);
    }

    #[test]
    fn read_bmp_with_image() {
        use image::GenericImageView;
        let a = image::open("test_bmp/test1.bmp").unwrap();
    }

    fn random_bmp() -> Bmp {
        let mut rng = rand::thread_rng();
        let width = rng.gen_range(1, 20);
        let height = rng.gen_range(1, 20);
        let data: Vec<bool> = (0..width * height).map(|_| rng.gen()).collect();
        Bmp::new(data, width).unwrap()
    }
}

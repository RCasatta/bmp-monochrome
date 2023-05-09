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

/// The `Bmp` struct contains the data as a vector of vectors of booleans.
/// Each boolean represent a pixel.
/// In `rows` the first element is the upper row, inside the first vector there are the pixel
/// from left to right, thus `rows[0][0]` is the upper-left element.
/// Max len of the vetors (both rows and colums) is [u16::MAX]`
/// Note in the serialized format the first element is the lower-left pixel
/// see [BMP file format](https://en.wikipedia.org/wiki/BMP_file_format)
#[derive(PartialEq, Eq, Clone)]
pub struct Bmp {
    rows: Vec<Vec<bool>>,
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
    Size(u16, u16),
}

impl Display for BmpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for Bmp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bmp width={} height={}", self.width(), self.height(),)
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
    height: u16,
    width: u16,
    bg_is_zero: bool,
}

impl Bmp {
    /// Creates a new Bmp, failing if `rows` is empty or it's first element is empty
    /// or it's elements has different len
    pub fn new(rows: Vec<Vec<bool>>) -> Result<Bmp, BmpError> {
        if rows.is_empty() || rows[0].is_empty() || !rows.iter().all(|e| e.len() == rows[0].len()) {
            Err(BmpError::Data)
        } else {
            check_size(u16::try_from(rows.len())?, u16::try_from(rows[0].len())?)?;
            Ok(Bmp { rows })
        }
    }

    /// return the Bmp height in pixel
    pub fn height(&self) -> u16 {
        self.rows.len() as u16
    }

    /// return the Bmp width in pixel
    pub fn width(&self) -> u16 {
        self.rows[0].len() as u16
    }

    /// return the pixel situated at (i,j), where (0,0) is the upper-left corner
    /// could panic if i > self.height() || j > self.width()
    pub fn get(&self, i: u16, j: u16) -> bool {
        self.rows[i as usize][j as usize]
    }

    /// return a new Bmp where every pixel is multiplied by `mul`, erroring if mul is 0 or 1 or the
    /// resulting image would be bigger than limits enforced by [crate::check_size]
    pub fn mul(&self, mul: u8) -> Result<Bmp, BmpError> {
        if mul <= 1 {
            return Err(BmpError::Generic);
        }
        let mul = mul as u16;
        let new_width = self
            .width()
            .checked_mul(mul)
            .ok_or_else(|| BmpError::Generic)?;
        let new_height = self
            .height()
            .checked_mul(mul)
            .ok_or_else(|| BmpError::Generic)?;
        check_size(new_width, new_height)?;
        let mut rows = Vec::with_capacity(new_height as usize);

        let mul = mul as usize;
        for i in 0..self.height() {
            let mut row = Vec::with_capacity(new_width as usize);
            for j in 0..self.width() {
                row.extend(vec![self.get(i, j); mul]);
            }
            rows.extend(vec![row; mul]);
        }

        Ok(Bmp { rows })
    }

    /// return a new Bmp where every square is divided by `div`
    /// if all the square is not of the same color it errors
    pub fn div(&self, div: u8) -> Result<Bmp, BmpError> {
        if div <= 1 {
            return Err(BmpError::Generic);
        }
        let div = div as u16;
        let new_height = self.height() / div;
        let new_width = self.width() / div;
        if new_height == 0 || new_width == 0 || self.height() % div != 0 || self.width() % div != 0
        {
            return Err(BmpError::Generic);
        }
        let mut new_rows = vec![];

        let div = div as usize;
        for rows in self.rows.chunks(div) {
            let mut new_row = vec![];
            for j in 0..div - 1 {
                if rows[j] != rows[j + 1] {
                    return Err(BmpError::Generic);
                }
            }
            for cols in rows[0].chunks(div) {
                if cols.iter().all(|e| cols[0] == *e) {
                    new_row.push(cols[0]);
                } else {
                    return Err(BmpError::Generic);
                }
            }
            new_rows.push(new_row);
        }
        Ok(Bmp { rows: new_rows })
    }

    fn div_with_greater_possible(&self, greater_start: u8) -> Bmp {
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
    pub fn add_white_border(&self, border_size: u8) -> Result<Bmp, BmpError> {
        let double_border = border_size as u16 * 2;
        let width = self
            .width()
            .checked_add(double_border)
            .ok_or_else(|| BmpError::Generic)?;
        let height = self
            .height()
            .checked_add(double_border)
            .ok_or_else(|| BmpError::Generic)?;
        check_size(width, height)?;
        let mut new_rows = Vec::with_capacity(height as usize);
        let border_size = border_size as usize;
        new_rows.extend(vec![vec![false; width as usize]; border_size]);
        for row in self.rows.iter() {
            let mut new_row = Vec::with_capacity(width as usize);
            new_row.extend(vec![false; border_size]);
            new_row.extend(row);
            new_row.extend(vec![false; border_size]);
            new_rows.push(new_row);
        }
        new_rows.extend(vec![vec![false; width as usize]; border_size]);

        Ok(Bmp { rows: new_rows })
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
        if self.width() <= 2 || self.height() <= 2 {
            return Err(BmpError::Generic);
        }
        let new_width = self.width() as usize - 2;
        let new_height = self.height() as usize - 2;
        let mut new_rows = vec![];
        if self.rows[0].iter().all(|e| !*e)
            && self.rows.last().unwrap().iter().all(|e| !*e)
            && self.rows.iter().all(|r| !r[0])
            && self.rows.iter().all(|r| !*r.last().unwrap())
        {
            for row in &self.rows[1..=new_height] {
                new_rows.push(row[1..=new_width].to_vec())
            }
            Ok(Bmp { rows: new_rows })
        } else {
            Err(BmpError::Generic)
        }
    }

    #[allow(dead_code)]
    fn to_test_string(&self) -> String {
        let mut s = String::new();
        for row in self.rows.iter() {
            for el in row.iter() {
                if *el {
                    s.push('#');
                } else {
                    s.push('.');
                }
            }
            s.push('\n');
        }
        s.trim_end().to_string()
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
        (self.width as u32 + 7) / 8
    }

    /// return the padding
    fn padding(&self) -> u32 {
        (4 - self.bytes_per_row() % 4) % 4
    }

    /// return wether the bit 0 is to be considered black
    fn bg_is_zero(&self) -> bool {
        self.bg_is_zero
    }
}

/// arbitrary limit width * height < 1 million
/// height and width must be > 0
fn check_size(width: u16, height: u16) -> Result<u32, BmpError> {
    let width_height = width as u32 * height as u32;
    if width_height <= 1_000_000 && width > 0 && height > 0 {
        Ok(width_height)
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
        assert!(Bmp::new(vec![]).is_err());
        assert!(Bmp::new(vec![vec![true]]).is_ok());
        assert!(Bmp::new(vec![vec![true], vec![true]]).is_ok());
        assert!(Bmp::new(vec![vec![true], vec![true, false]]).is_err());
    }

    #[test]
    fn test_padding() {
        let mut header = BmpHeader {
            height: 0,
            width: 0,
            bg_is_zero: false,
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
            bg_is_zero: false,
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
        let data = Bmp::new(vec![vec![false, true], vec![false, true]]).unwrap();

        let data_bigger = Bmp::new(vec![
            vec![false, false, true, true],
            vec![false, false, true, true],
            vec![false, false, true, true],
            vec![false, false, true, true],
        ])
        .unwrap();

        assert_eq!(data.mul(2).unwrap(), data_bigger);

        let data = Bmp::new(vec![vec![false, true], vec![false, false]]).unwrap();

        let data_bigger = Bmp::new(vec![
            vec![false, false, true, true],
            vec![false, false, true, true],
            vec![false, false, false, false],
            vec![false, false, false, false],
        ])
        .unwrap();

        assert_eq!(data.mul(2).unwrap(), data_bigger);
    }

    #[test]
    fn test_div() {
        let data = Bmp::new(vec![
            vec![false, false, true, true],
            vec![false, false, true, true],
            vec![false, false, true, true],
            vec![false, false, true, true],
        ])
        .unwrap();
        let expected = Bmp::new(vec![vec![false, true], vec![false, true]]).unwrap();
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
        let data = Bmp::new(vec![vec![false]]).unwrap();
        let data_bigger = Bmp::new(vec![vec![false; 5]; 5]).unwrap();

        assert_eq!(data.add_white_border(2).unwrap(), data_bigger);
    }

    #[test]
    fn test_rect() {
        let rect = Bmp::new(vec![
            vec![false, false],
            vec![false, false],
            vec![false, true],
        ])
        .unwrap();
        rect.write(File::create("test_bmp/rect.bmp").unwrap())
            .unwrap();
    }

    #[test]
    fn test_bmp() {
        let data_test1 = Bmp::new(vec![vec![false, true], vec![true, false]]).unwrap();
        let bytes_test1 = Bmp::read(&mut File::open("test_bmp/test1.bmp").unwrap()).unwrap();
        assert_eq!(data_test1, bytes_test1);

        let bmp_test2 = data_test1.mul(3).unwrap().add_white_border(12).unwrap();
        let bytes_test2 = Bmp::read(&mut File::open("test_bmp/test2.bmp").unwrap()).unwrap();
        assert_eq!(bmp_test2.to_test_string(), bytes_test2.to_test_string());
    }


    #[test]
    fn test_bmp_with_bg_is_zero() {
        let bmp = Bmp::read(&mut File::open("test_bmp/qr-bolt11.bmp").unwrap()).unwrap();
        let mut cursor = Cursor::new(vec![]);
        bmp.write(&mut cursor).unwrap();
        cursor.set_position(0);
        let bmp2 = Bmp::read(cursor).unwrap();
        assert_eq!(bmp, bmp2);
    }

    #[test]
    fn test_monochrome_image() {
        // taken from https://github.com/pertbanking/bitmap-monochrome/blob/master/monochrome_image.bmp
        let expected = Bmp::new(
            vec![
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
            .chunks(18)
            .map(|r| r.iter().map(|e| *e == 0).collect())
            .collect(),
        )
        .unwrap();

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
    fn test_get() {
        let file = File::open("test_bmp/test1.bmp").unwrap();
        let bmp = Bmp::read(file).unwrap();
        assert!(!bmp.get(0, 0), "lower-left is not dark");
    }

    #[test]
    fn test_remove_white_border() {
        let bmp5 = Bmp::new(vec![vec![false; 5]; 5]).unwrap();
        let bmp3 = Bmp::new(vec![vec![false; 3]; 3]).unwrap();
        assert_eq!(bmp3, bmp5.remove_one_white_border().unwrap());
        let bmp1 = Bmp::new(vec![vec![false]]).unwrap();
        assert_eq!(bmp1, bmp3.remove_one_white_border().unwrap());
        assert_eq!(bmp1, bmp5.remove_white_border());
    }

    #[test]
    fn test_div_with_greater_possible() {
        let bmp = Bmp::read(File::open("test_bmp/monochrome_image.bmp").unwrap()).unwrap();
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

    fn random_bmp() -> Bmp {
        let mut rng = rand::thread_rng();
        let width: u16 = rng.gen_range(1, 20);
        let height: u16 = rng.gen_range(1, 20);
        let mut data = vec![];
        for _ in 0..height {
            let row: Vec<bool> = (0..width).map(|_| rng.gen()).collect();
            data.push(row);
        }
        Bmp::new(data).unwrap()
    }
}

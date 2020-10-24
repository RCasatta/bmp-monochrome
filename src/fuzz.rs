//! fuzzing!

use crate::{check_size, Bmp, BmpError};
use arbitrary::Arbitrary;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba};
use std::io::Cursor;

impl arbitrary::Arbitrary for Bmp {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let width = u32::arbitrary(u)?;
        let height = u32::arbitrary(u)?;
        check_size(width, height).map_err(|_| arbitrary::Error::IncorrectFormat)?;
        let total = (width * height) as usize;
        let mut data = Vec::with_capacity(total);
        for _ in 0..total {
            data.push(bool::arbitrary(u)?);
        }
        let bmp = Bmp::new(data, width as usize).unwrap();
        Ok(bmp)
    }
}

#[derive(Debug, Arbitrary)]
/// Possible operations called on a Bmp, used for fuzz tests
pub enum Op {
    /// bmp.mul
    Mul(usize),
    /// bmp.div
    Div(usize),
    /// bmp.add_white_border
    Border(usize),
    /// bmp.remove_white_border
    RemoveBorder,
    /// bmp.normalize
    Normalize,
}

/// Used for fuzz testing creating a random Bmp and a random Op to apply to
#[derive(Debug, Arbitrary)]
pub struct BmpAndOps {
    /// the Bmp
    pub bmp: Bmp,
    /// the operation to perform
    pub ops: Vec<Op>,
}

impl Bmp {
    /// check that image crate loads the same pixel
    pub fn check(&self) {
        if self.width > 10000 || self.height() > 10000 {
            return;
        }
        let mut cursor = Cursor::new(vec![]);
        self.write(&mut cursor).unwrap();
        cursor.set_position(0);
        let image =
            image::load_from_memory_with_format(&cursor.into_inner(), ImageFormat::Bmp).unwrap();

        let (width, height) = image.dimensions();
        assert_eq!(width, self.width as u32);
        assert_eq!(height, self.height() as u32);
        assert_eq!(to_test_string(&image), self.to_test_string());
    }

    fn to_test_string(&self) -> String {
        let mut s = String::new();
        for i in 0..self.width {
            for j in 0..self.height() {
                if self.get(j, i) {
                    s.push('#');
                } else {
                    s.push('.');
                }
            }
            if i < self.width - 1 {
                s.push('\n');
            }
        }
        s
    }
}

fn to_test_string(image: &DynamicImage) -> String {
    let (width, height) = image.dimensions();
    let mut s = String::new();
    for i in (0..width) {
        for j in (0..height) {
            match image.get_pixel(i, j) {
                Rgba([255, 255, 255, 255]) => {
                    s.push('.');
                }
                Rgba([0, 0, 0, 255]) => {
                    s.push('#');
                }
                _ => assert!(false),
            }
        }

        s.push('\n');
    }

    s.trim().to_string()
}

impl BmpAndOps {
    /// apply operation on this bmp
    pub fn apply(self) -> Result<(), BmpError> {
        let BmpAndOps { mut bmp, ops } = self;
        for op in ops {
            bmp = match op {
                Op::Mul(mul) => bmp.mul(mul)?,
                Op::Div(div) => bmp.div(div)?,
                Op::Border(border) => bmp.add_white_border(border)?,
                Op::Normalize => bmp.normalize(),
                Op::RemoveBorder => bmp.remove_white_border(),
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::fuzz::BmpAndOps;
    use crate::Bmp;
    use arbitrary::Arbitrary;
    use std::fs::File;

    #[test]
    fn test_bmp_check() {
        let bmp = Bmp::read(File::open("test_bmp/rect.bmp").unwrap()).unwrap();
        bmp.check();
    }

    #[test]
    fn test_bmp_check_fuzz() {
        let data = base64::decode("PjYAADY=").unwrap();
        let unstructured = arbitrary::Unstructured::new(&data[..]);
        let data = Bmp::arbitrary_take_rest(unstructured).unwrap();
        dbg!(&data);
        data.check();
    }

    #[test]
    fn test_bmp_and_ops() {
        //let data = base64::decode("AQAAAAEAAAACQIAAABUFAAAAAAAAlZ2dnQAAAAAAAAA0").unwrap();
        let data = base64::decode("AQAAAAgAAAAAAAAAAAAAAAAACgoAAAAKGgAAAAAA").unwrap();
        let unstructured = arbitrary::Unstructured::new(&data[..]);
        let data = BmpAndOps::arbitrary_take_rest(unstructured).unwrap();
        dbg!(&data);
        let _ = data.apply();
    }
}

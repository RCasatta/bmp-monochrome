//! fuzzing!

use crate::{check_size, Bmp, BmpError};
use arbitrary::Arbitrary;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba};
use std::io::Cursor;

impl arbitrary::Arbitrary for Bmp {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let width = u16::arbitrary(u)?;
        let height = u16::arbitrary(u)?;
        check_size(width, height).map_err(|_| arbitrary::Error::IncorrectFormat)?;
        let mut rows = Vec::with_capacity(height as usize);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width as usize);
            for _ in 0..width {
                row.push(bool::arbitrary(u)?);
            }
            rows.push(row);
        }
        let bmp = Bmp::new(rows).unwrap();
        Ok(bmp)
    }
}

#[derive(Debug, Arbitrary)]
/// Possible operations called on a Bmp, used for fuzz tests
pub enum Op {
    /// bmp.mul
    Mul(u8),
    /// bmp.div
    Div(u8),
    /// bmp.add_white_border
    Border(u8),
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
    /// the 4 operations to perform
    pub ops: [Op; 4],
}

impl Bmp {
    /// check that image crate loads the same pixel
    pub fn check(&self) {
        let mut cursor = Cursor::new(vec![]);
        self.write(&mut cursor).unwrap();
        cursor.set_position(0);
        let image =
            image::load_from_memory_with_format(&cursor.into_inner(), ImageFormat::Bmp).unwrap();

        let (width, height) = image.dimensions();
        assert_eq!(width, self.width() as u32);
        assert_eq!(height, self.height() as u32);
        assert_eq!(to_test_string(&image), self.to_test_string());
    }
}

fn rgba_to_bool(pixel: Rgba<u8>) -> bool {
    match pixel {
        Rgba([0, 0, 0, 255]) => true,
        _ => false,
    }
}

fn to_test_string(image: &DynamicImage) -> String {
    let mut s = String::new();
    for (i, j, pixel) in image.pixels() {
        if i == 0 && j != 0 {
            s.push('\n');
        }
        if rgba_to_bool(pixel) {
            s.push('#');
        } else {
            s.push('.');
        }
    }
    s
}

impl BmpAndOps {
    /// apply operation on this bmp
    pub fn apply(self) -> Result<(), BmpError> {
        let BmpAndOps { mut bmp, ops } = self;
        for op in ops.iter() {
            bmp = match op {
                Op::Mul(mul) => bmp.mul(*mul)?,
                Op::Div(div) => bmp.div(*div)?,
                Op::Border(border) => bmp.add_white_border(*border)?,
                Op::Normalize => bmp.normalize(),
                Op::RemoveBorder => bmp.remove_white_border(),
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_bmp_check() {
        let bmp = crate::Bmp::read(std::fs::File::open("test_bmp/rect.bmp").unwrap()).unwrap();
        bmp.check();
    }

    /*
    #[test]
    fn test_bmp_check_fuzz() {
        let data = base64::decode("PjYAADY=").unwrap();
        let unstructured = arbitrary::Unstructured::new(&data[..]);
        let data = Bmp::arbitrary_take_rest(unstructured).unwrap();
        dbg!(&data);
        data.check();
    }
    */

    /*
    #[test]
    fn test_bmp_and_ops() {
        use crate::fuzz::BmpAndOps;
        //let data = base64::decode("AQAAAAEAAAACQIAAABUFAAAAAAAAlZ2dnQAAAAAAAAA0").unwrap();
        //let data = base64::decode("AwABAAAAAAAAAAEl9v8A//8A").unwrap();
        let data =
            include_bytes!("../fuzz/artifacts/ops/oom-1c626462ce8bc20e025a4fd7f0b6e2e513dac895");
        let unstructured = arbitrary::Unstructured::new(&data[..]);
        let data = BmpAndOps::arbitrary_take_rest(unstructured).unwrap();
        dbg!(&data);
        let _ = data.apply();
    }
    */
}

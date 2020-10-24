//! fuzzing!

use crate::{check_size, Bmp, BmpError};
use arbitrary::Arbitrary;

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
    use arbitrary::Arbitrary;

    #[test]
    fn test_fuzz() {
        let data = base64::decode("AQAAAAgAAAAKAQAAAAAAAAEIAAIAAAAKGgAAKQAaAAApAAAA").unwrap();
        let unstructured = arbitrary::Unstructured::new(&data[..]);
        let data = BmpAndOps::arbitrary_take_rest(unstructured);
        let _ = data.unwrap().apply();
    }
}

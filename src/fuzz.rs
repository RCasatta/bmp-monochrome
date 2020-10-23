//! fuzzing!

use crate::{check_size, Bmp};
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
/// Op
pub enum Op {
    /// Mul
    Mul(usize),
    /// Div
    Div(usize),
    /// Border
    Border(usize),
    /// RemoveBorder
    RemoveBorder,
    /// Normalize
    Normalize,
}

/// ooo
#[derive(Debug, Arbitrary)]
pub struct BmpAndOp {
    /// bmp
    pub bmp: Bmp,
    /// ops
    pub op: Op,
}

impl BmpAndOp {
    /// apply operations on this bmp
    pub fn apply(self) {
        let bmp = self.bmp.clone();
        let _ = match self.op {
            Op::Mul(mul) => bmp.mul(mul),
            Op::Div(div) => bmp.div(div),
            Op::Border(border) => bmp.add_white_border(border),
            Op::Normalize => Ok(bmp.normalize()),
            Op::RemoveBorder => Ok(bmp.remove_white_border()),
        };
    }
}

#[cfg(test)]
mod test {
    use crate::fuzz::BmpAndOp;
    use crate::Bmp;
    use arbitrary::Arbitrary;

    #[test]
    fn test_fuzz() {
        let data = base64::decode("AAEAAC0=").unwrap();
        dbg!(&data);
        //let data = include_bytes!("../test_bmp/crash-091bf790e1922d7008ca0f9b3b19cb3106fad41b");
        let mut unstructured = arbitrary::Unstructured::new(&data[..]);
        let data = BmpAndOp::arbitrary(&mut unstructured);
        dbg!(&data);
        data.unwrap().apply();
    }
}

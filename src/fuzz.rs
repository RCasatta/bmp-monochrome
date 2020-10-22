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
/// Ops
pub enum Ops {
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
pub struct BmpAndOps {
    /// bmp
    pub bmp: Bmp,
    /// ops
    pub ops: Vec<Ops>,
}

#[cfg(test)]
mod test {
    use crate::fuzz::Ops;
    use crate::Bmp;

    #[test]
    fn test_fuzz() {
        use arbitrary::Arbitrary;
        //let data = base64::decode("BQAAAAEAAAAAAAQAAAADAwAAAAAAAKysrKysrKysrKysrKz//////w==").unwrap();
        let data = include_bytes!("../test_bmp/crash-dda9ce37b68d23a3bd61bf074ebb4c9fd24c91b7");
        let mut unstructured = arbitrary::Unstructured::new(&data[..]);
        let data: (Bmp, Vec<Ops>) = Arbitrary::arbitrary(&mut unstructured).unwrap();
        dbg!(&data);
        let _a = data.0.remove_white_border();
        //assert!(a.is_err());
    }
}

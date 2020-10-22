#![no_main]
use bmp_monochrome::fuzz::{BmpAndOps, Ops};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: BmpAndOps| {
    let mut bmp = data.bmp.clone();
    for op in data.ops {
        bmp = match op {
            Ops::Mul(mul) => match bmp.mul(mul) {
                Ok(bmp) => bmp,
                Err(_) => break,
            },
            Ops::Div(div) => match bmp.div(div) {
                Ok(bmp) => bmp,
                Err(_) => break,
            },
            Ops::Border(border) => match bmp.add_white_border(border) {
                Ok(bmp) => bmp,
                Err(_) => break,
            },
            Ops::Normalize => bmp.normalize(),
            Ops::RemoveBorder => bmp.remove_white_border(),
        }
    }
});

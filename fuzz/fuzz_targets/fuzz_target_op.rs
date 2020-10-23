#![no_main]
use bmp_monochrome::fuzz::BmpAndOp;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: BmpAndOp| {
    data.apply();
});

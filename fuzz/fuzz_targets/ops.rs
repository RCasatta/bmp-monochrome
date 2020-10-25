#![no_main]
use bmp_monochrome::fuzz::BmpAndOps;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: BmpAndOps| {
    let _ = data.apply();
});

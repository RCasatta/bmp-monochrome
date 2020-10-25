#![no_main]
use libfuzzer_sys::fuzz_target;
use bmp_monochrome::Bmp;

fuzz_target!(|bmp: Bmp| {
    bmp.check();
});

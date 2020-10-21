#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|bmp: bmp_monochrome::Bmp| {
    let _ = bmp.write(Cursor::new(vec![]));
});

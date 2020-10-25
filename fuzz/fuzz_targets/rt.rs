#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;
use bmp_monochrome::Bmp;

fuzz_target!(|bmp: bmp_monochrome::Bmp| {
    let mut cursor = Cursor::new(vec![]);
    let _ = bmp.write(&mut cursor).unwrap();
    cursor.set_position(0);
    let bmp_read = Bmp::read(&mut cursor).unwrap();
    assert_eq!(bmp, bmp_read);
});

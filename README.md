# bmp-monochrome

[![crates.io](https://img.shields.io/crates/v/bmp-monochrome.svg)](https://crates.io/crates/bmp-monochrome)

Encode and decode monochromatic bitmaps without additional dependencies, useful for QR codes.

## Example

```rust
use bmp_monochrome::Bmp;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "test.bmp";
    let width = 21;
    let data: Vec<bool> = (0..width * width).map(|e| e % 2 == 0).collect();
    let rows: Vec<Vec<bool>> = data.chunks(width).map(|e| e.to_vec()).collect();
    let bmp = Bmp::new(rows)?;
    bmp.write(File::create(file_name)?)?;
    let bmp_read = Bmp::read(File::open(file_name)?)?;
    assert_eq!(bmp, bmp_read);
    Ok(())
}
```

Generates

![test](https://raw.githubusercontent.com/RCasatta/bmp-monochrome/master/test.bmp)

## Minimum Supported Rust Version (MSRV)

*Rust 1.34*

Use
 [u32::try_from](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) introduced in 1.34.0
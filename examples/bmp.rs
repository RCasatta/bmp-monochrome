use bmp_monochrome::Bmp;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "test.bmp";
    let width = 21;
    let data: Vec<bool> = (0..width * width).map(|e| e % 2 == 0).collect();
    let bmp = Bmp::new(data, width)?;
    bmp.write(File::create(file_name)?)?;
    let bmp_read = Bmp::read(File::open(file_name)?)?;
    assert_eq!(bmp, bmp_read);
    Ok(())
}

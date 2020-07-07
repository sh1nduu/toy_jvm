use std::error::Error;
use std::fs::File;
use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

struct Loader {
    cursor: Cursor<Vec<u8>>
}

#[allow(dead_code)]
impl Loader {
    fn new(file: &mut File) -> Self {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let cursor = Cursor::new(buf);
        Self { cursor }
    }
    fn u1(&mut self) -> u8 {
        self.cursor.read_u8().unwrap()
    }
    fn u2(&mut self) -> u16 {
        self.cursor.read_u16::<BigEndian>().unwrap()
    }
    fn u4(&mut self) -> u32 {
        self.cursor.read_u32::<BigEndian>().unwrap()
    }
    fn u8(&mut self) -> u64 {
        self.cursor.read_u64::<BigEndian>().unwrap()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("Add.class")?;
    let mut loader = Loader::new(&mut file);
    let cafebabe = loader.u4();
    let major = loader.u2();
    let minor = loader.u2();

    println!("cafebabe {}, major {}, minor {}", cafebabe, major, minor);
    Ok(())
}

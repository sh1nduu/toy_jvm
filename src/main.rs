use byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;

#[derive(Debug)]
struct Const {
    pub tag: u8,
    pub name_index: Option<u16>,
    pub class_index: Option<u16>,
    pub name_and_type_index: Option<u16>,
    pub string_index: Option<u16>,
    pub desc_index: Option<u16>,
    pub string: Option<String>,
}
impl Const {
    fn new(tag: u8) -> Self {
        Self {
            tag,
            name_index: None,
            class_index: None,
            name_and_type_index: None,
            string_index: None,
            desc_index: None,
            string: None,
        }
    }
}

#[derive(Debug)]
struct ConstPool(Vec<Const>);

struct Loader {
    cursor: Cursor<Vec<u8>>,
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
    fn bytes(&mut self, size: usize) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; size];
        self.cursor.read_exact(&mut buf).unwrap();
        buf
    }
    fn cpinfo(&mut self) -> ConstPool {
        let const_pool_count = self.u2();
        let mut const_pool = Vec::new();
        for _ in 1..const_pool_count {
            let mut c = Const::new(self.u1());
            match c.tag {
                0x01 => {
                    // UTF-8 string literal, 2 bytes lenght + data
                    let size = self.u2();
                    let bytes = self.bytes(size.into());
                    let string = String::from_utf8(bytes).unwrap();
                    c.string = Some(string);
                }
                0x07 => c.name_index = Some(self.u2()),
                0x08 => c.string_index = Some(self.u2()),
                0x09 | 0x0a => {
                    c.class_index = Some(self.u2());
                    c.name_and_type_index = Some(self.u2());
                }
                0x0c => {
                    c.name_index = Some(self.u2());
                    c.desc_index = Some(self.u2());
                }
                n => println!("unsupported tag {}", n),
            }
            const_pool.push(c);
        }

        ConstPool(const_pool)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("Add.class")?;
    let mut loader = Loader::new(&mut file);
    let cafebabe = loader.u4();
    let major = loader.u2();
    let minor = loader.u2();

    println!("cafebabe {}, major {}, minor {}", cafebabe, major, minor);

    let const_pool = loader.cpinfo();

    println!("{:?}", const_pool);
    Ok(())
}

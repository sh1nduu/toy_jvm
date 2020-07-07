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

impl ConstPool {
    fn resolve(&self, index: usize) -> Option<String> {
        if self.0[index - 1].tag == 0x01 {
            self.0[index - 1].string.clone()
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Field {
    flags: u16,
    name: String,
    descriptor: String,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    data: Vec<u8>,
}
struct Loader {
    cursor: Cursor<Vec<u8>>,
}

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
    fn interfaces(&mut self, cp: &ConstPool) -> Vec<String> {
        (0..self.u2()).filter_map(|_| cp.resolve(self.u2() as usize)).collect()
    }
    fn fields(&mut self, cp: &ConstPool) -> Vec<Field> {
        (0..self.u2()).map(|_| Field {
            flags: self.u2(),
            name: cp.resolve(self.u2() as usize).unwrap_or("".into()),
            descriptor: cp.resolve(self.u2() as usize).unwrap_or("".into()),
            attributes: self.attrs(cp),
        }).collect()
    }
    fn attrs(&mut self, cp: &ConstPool) -> Vec<Attribute> {
        (0..self.u2()).map(|_| {
            let name = cp.resolve(self.u2() as usize).unwrap_or("".into());
            let data_size = self.u4() as usize;
            let data = self.bytes(data_size);
            Attribute { name, data }
        }).collect()
    }
}

#[derive(Debug)]
struct Class {
    const_pool: ConstPool,
    name: String,
    super_: String,
    flags: u16,
    interfaces: Vec<String>,
    fields: Vec<Field>,
    methods: Vec<Field>,
    attributes: Vec<Attribute>,
}

impl Class {
    fn new(file: &mut File) -> Self {
        let mut loader = Loader::new(file);
        loader.u8();
        let const_pool = loader.cpinfo();
        let flags = loader.u2();
        let name = const_pool.resolve(loader.u2() as usize).unwrap_or("".into());
        let super_ = const_pool.resolve(loader.u2() as usize).unwrap_or("".into());
        let interfaces = loader.interfaces(&const_pool);
        let fields = loader.fields(&const_pool);
        let methods = loader.fields(&const_pool);
        let attributes = loader.attrs(&const_pool);
        Class { const_pool, flags, name, super_, interfaces, fields, methods, attributes }
    }

    fn frame(&self, method: String, args: Vec<i32>) -> Option<Frame> {
        for m in &self.methods {
            if m.name == method {
                for a in &m.attributes {
                    if a.name == "Code" && a.data.len() > 8 {
                        return Some(Frame {
                            class: self,
                            ip: 0,
                            code: a.data[8..].to_vec(),
                            locals: args,
                            stack: Vec::new(),
                        });
                    }
                }
            }
        }
        None
    }
}

#[allow(dead_code)]
struct Frame<'a> {
    class: &'a Class,
    ip: u32,
    code: Vec<u8>,
    locals: Vec<i32>,
    stack: Vec<i32>,
}

fn exec(frame: &mut Frame) -> i32 {
    loop {
        let op = frame.code[frame.ip as usize];
        println!("OP: {:>02x} {:?}", op, frame.stack);
        match op {
            // iload_0
            26 => frame.stack.push(frame.locals[0]),
            // iload_1
            27 => frame.stack.push(frame.locals[1]),
            // iadd
            96 => {
                match (frame.stack.pop(), frame.stack.pop()) {
                    (Some(a), Some(b)) => frame.stack.push(a + b),
                    _ => panic!("Arguments number does not match"),
                }
            }
            // ireturn
            172 => {
                if let Some(v) = frame.stack.pop() {
                    return v;
                } else {
                    panic!("No return values");
                }
            }
            n => panic!("unsupported operator {:?}", n)
        }
        frame.ip += 1;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("Add.class")?;
    let class = Class::new(&mut file);
    let mut frame = class.frame("add".into(), vec![2, 3]).unwrap();
    let result = exec(&mut frame);
    println!("{:?}", result);
    Ok(())
}

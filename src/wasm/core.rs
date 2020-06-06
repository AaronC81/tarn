use leb128;

pub trait WasmCodeGen {
    fn generate_wasm(&self) -> Vec<u8>;

    fn generate_wasm_seq<T: WasmCodeGen>(&self, items: &Vec<T>) -> Vec<u8> {
        items.iter().map(|x| x.generate_wasm()).flatten().collect()
    }

    fn generate_wasm_vec<T: WasmCodeGen>(&self, items: &Vec<T>) -> Vec<u8> {
        [
            encode_u32(items.len() as u32),
            self.generate_wasm_seq(items),
        ].concat()
    }
}

pub fn encode_u32(n: u32) -> Vec<u8> {
    let mut buf = vec![];
    leb128::write::unsigned(&mut buf, n as u64).expect("leb128 conversion failed");
    buf
}

pub fn encode_i32(n: i32) -> Vec<u8> {
    let mut buf = vec![];
    leb128::write::signed(&mut buf, n as i64).expect("leb128 conversion failed");
    buf
}

impl WasmCodeGen for String {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            encode_u32(self.len() as u32),
            self.bytes().collect(),
        ].concat()
    }
}

impl WasmCodeGen for u32 {
    fn generate_wasm(&self) -> Vec<u8> {
        encode_u32(*self)
    }
}

#[derive(Debug, Clone)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    pub fn to_byte(&self) -> u8 {
        match self {
            ValueType::I32 => 0x7F,
            ValueType::I64 => 0x7E,
            ValueType::F32 => 0x7D,
            ValueType::F64 => 0x7C,
        }
    }
}

impl WasmCodeGen for ValueType {
    fn generate_wasm(&self) -> Vec<u8> {
        vec![self.to_byte()]
    }
}

pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl WasmCodeGen for Limits {
    fn generate_wasm(&self) -> Vec<u8> {
        if let Some(max) = self.max {
            [
                vec![0x01],
                encode_u32(self.min),
                encode_u32(max),
            ].concat()
        } else {
            [
                vec![0x00],
                encode_u32(self.min),
            ].concat()
        }
    }
}

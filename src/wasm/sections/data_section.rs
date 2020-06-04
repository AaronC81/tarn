use super::BodySection;
use crate::wasm::core::{WasmCodeGen, encode_u32};
use crate::wasm::instruction::Expr;

pub struct DataSection {
    pub data: Vec<Data>,
}

impl BodySection for DataSection {
    const ID: u8 = 11;
    type BodyItem = Data;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.data }
}

pub struct Data {
    pub memory: u32,
    pub expr: Expr,
    pub init: Vec<u8>,
}

impl WasmCodeGen for Data {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            encode_u32(self.memory),
            self.expr.generate_wasm(),
            encode_u32(self.init.len() as u32),
            self.init.clone(),
        ].concat()
    }
}

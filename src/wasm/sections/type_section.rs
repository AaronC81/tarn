use super::BodySection;
use crate::wasm::core::{WasmCodeGen, ValueType};

pub struct TypeSection {
    pub func_types: Vec<FuncType>,
}

impl BodySection for TypeSection {
    const ID: u8 = 1;
    type BodyItem = FuncType;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.func_types }
}

pub struct FuncType {
    pub parameters: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl WasmCodeGen for FuncType {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            vec![0x60],
            self.generate_wasm_vec(&self.parameters),
            self.generate_wasm_vec(&self.results),
        ].concat()
    }
}

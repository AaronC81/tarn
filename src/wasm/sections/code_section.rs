use super::BodySection;
use crate::wasm::core::{WasmCodeGen, encode_u32, ValueType};
use crate::wasm::instruction::Expr;

pub struct CodeSection {
    pub codes: Vec<Code>,
}

impl BodySection for CodeSection {
    const ID: u8 = 10;
    type BodyItem = Code;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.codes }
}

pub struct Code {
    pub func: Func,
}

impl WasmCodeGen for Code {
    fn generate_wasm(&self) -> Vec<u8> {
        let func = self.func.generate_wasm();
        [
            encode_u32(func.len() as u32),
            func,
        ].concat()
    }
}

pub struct Func {
    pub locals: Vec<Local>,
    pub expr: Expr,
}

impl WasmCodeGen for Func {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.generate_wasm_vec(&self.locals),
            self.expr.generate_wasm(),
        ].concat()
    }
}

pub struct Local {
    pub n: u32,
    pub value_type: ValueType,
}

impl WasmCodeGen for Local {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            encode_u32(self.n),
            vec![self.value_type.to_byte()],
        ].concat()
    }
}

use super::BodySection;
use crate::wasm::core::{WasmCodeGen, encode_u32};

pub struct ImportSection {
    pub imports: Vec<Import>,
}

impl BodySection for ImportSection {
    const ID: u8 = 2;
    type BodyItem = Import;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.imports }
}

pub struct Import {
    pub module: String,
    pub name: String,
    pub desc: ImportDesc,
}

impl WasmCodeGen for Import {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.module.generate_wasm(),
            self.name.generate_wasm(),
            self.desc.generate_wasm(),
        ].concat()
    }
}

pub enum ImportDesc {
    // TODO: other import types
    Func(u32),
}

impl WasmCodeGen for ImportDesc {
    fn generate_wasm(&self) -> Vec<u8> {
        match self {
            ImportDesc::Func(i) => [
                vec![0x00],
                encode_u32(*i),
            ].concat()
        }
    }
}

use super::BodySection;
use crate::wasm::core::{WasmCodeGen, encode_u32};

pub struct ExportSection {
    pub exports: Vec<Export>,
}

impl BodySection for ExportSection {
    const ID: u8 = 7;
    type BodyItem = Export;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.exports }
}

pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

impl WasmCodeGen for Export {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.name.generate_wasm(),
            self.desc.generate_wasm(),
        ].concat()
    }
}

pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Mem(u32),
    Global(u32),
}

impl ExportDesc {
    fn type_byte(&self) -> u8 {
        match self {
            ExportDesc::Func(_) => 0,
            ExportDesc::Table(_) => 1,
            ExportDesc::Mem(_) => 2,
            ExportDesc::Global(_) => 3,
        }
    }
}


impl WasmCodeGen for ExportDesc {
    fn generate_wasm(&self) -> Vec<u8> {

        match self {
            ExportDesc::Func(i) |
            ExportDesc::Table(i) |
            ExportDesc::Mem(i) |
            ExportDesc::Global(i) => [
                vec![self.type_byte()],
                encode_u32(*i),
            ].concat()
        }
    }
}

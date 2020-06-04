use super::BodySection;
use crate::wasm::core::{WasmCodeGen, Limits};

pub struct MemorySection {
    pub memories: Vec<Memory>,
}

impl BodySection for MemorySection {
    const ID: u8 = 5;
    type BodyItem = Memory;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.memories }
}

pub struct Memory {
    pub memory_type: Limits,
}

impl WasmCodeGen for Memory {
    fn generate_wasm(&self) -> Vec<u8> {
        self.memory_type.generate_wasm()
    }
}

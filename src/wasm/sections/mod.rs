use crate::wasm::core::{WasmCodeGen, encode_u32};

pub mod type_section;
pub use type_section::TypeSection;

pub mod import_section;
pub use import_section::ImportSection;

pub mod function_section;
pub use function_section::FunctionSection;

pub mod memory_section;
pub use memory_section::MemorySection;

pub mod code_section;
pub use code_section::CodeSection;

pub mod export_section;
pub use export_section::ExportSection;

pub mod data_section;
pub use data_section::DataSection;

pub trait Section {
    const ID: u8;
}

pub trait BodySection {
    const ID: u8;
    type BodyItem : WasmCodeGen;

    fn body_item(&self) -> &Vec<Self::BodyItem>;
}

impl<T : BodySection> Section for T {
    const ID: u8 = Self::ID;
}

impl<T : BodySection> WasmCodeGen for T {
    fn generate_wasm(&self) -> Vec<u8> {
        let body = self.generate_wasm_vec(&self.body_item());
        [
            vec![Self::ID],
            encode_u32(body.len() as u32),
            body,
        ].concat()
    }
}

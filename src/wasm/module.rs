use super::core::WasmCodeGen;
use super::sections::{TypeSection, ImportSection, FunctionSection, MemorySection, CodeSection, ExportSection, DataSection};

pub struct Module {
    pub type_sections: Vec<TypeSection>,
    pub import_sections: Vec<ImportSection>,
    pub function_sections: Vec<FunctionSection>,
    pub memory_sections: Vec<MemorySection>,
    pub export_sections: Vec<ExportSection>,
    pub code_sections: Vec<CodeSection>,
    pub data_sections: Vec<DataSection>,
}

impl Module {
    fn magic(&self) -> Vec<u8> {
        vec![0x00, 0x61, 0x73, 0x6D]
    }

    fn version(&self) -> Vec<u8> {
        vec![0x01, 0x00, 0x00, 0x00]
    }
}

impl WasmCodeGen for Module {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.magic(),
            self.version(),
            self.generate_wasm_seq(&self.type_sections),
            self.generate_wasm_seq(&self.import_sections),
            self.generate_wasm_seq(&self.function_sections),
            self.generate_wasm_seq(&self.memory_sections),
            self.generate_wasm_seq(&self.export_sections),
            self.generate_wasm_seq(&self.code_sections),
            self.generate_wasm_seq(&self.data_sections),
        ].concat()
    }
}

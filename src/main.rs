mod wasm;

use std::fs::File;
use std::io::Write;

use crate::wasm::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module = Module {
        memory_sections: vec![
            MemorySection {
                memories: vec![
                    Memory {
                        memory_type: Limits {
                            min: 1,
                            max: None,
                        }
                    }
                ]
            }
        ],
        import_sections: vec![
            ImportSection {
                imports: vec![
                    Import {
                        module: "wasi_unstable".into(),
                        name: "fd_write".into(),
                        desc: ImportDesc::Func(1),
                    }
                ]
            }
        ],
        type_sections: vec![
            TypeSection {
                func_types: vec![
                    FuncType {
                        parameters: vec![],
                        results: vec![],
                    },
                    FuncType {
                        parameters: vec![
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        results: vec![
                            ValueType::I32,
                        ]
                    },
                ]
            }
        ],
        function_sections: vec![
            FunctionSection {
                types: vec![0],
            }
        ],
        export_sections: vec![
            ExportSection {
                exports: vec![
                    Export {
                        name: "main".into(),
                        desc: ExportDesc::Func(0),
                    }
                ]
            }
        ],
        code_sections: vec![
            CodeSection {
                codes: vec![
                    Code {
                        func: Func {
                            locals: vec![],
                            expr: Expr {
                                instructions: vec![
                                    Instruction::I32Const(81),
                                    Instruction::Drop,
                                ]
                            },
                        }
                    }
                ]
            }
        ]
    };
    let wasm = module.generate_wasm();

    let mut f = File::create("out.wasm")?;
    f.write_all(&wasm[..])?;

    Ok(())
}

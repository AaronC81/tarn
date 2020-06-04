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
                        desc: ImportDesc::Func(0),
                    }
                ]
            }
        ],
        type_sections: vec![
            TypeSection {
                func_types: vec![
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
                    FuncType {
                        parameters: vec![],
                        results: vec![],
                    },
                ]
            }
        ],
        function_sections: vec![
            FunctionSection {
                types: vec![1],
            }
        ],
        export_sections: vec![
            ExportSection {
                exports: vec![
                    Export {
                        name: "_start".into(),
                        desc: ExportDesc::Func(1),
                    },
                    Export {
                        name: "memory".into(),
                        desc: ExportDesc::Mem(0),
                    }
                ]
            }
        ],
        data_sections: vec![
            DataSection {
                data: vec![
                    Data {
                        memory: 0,
                        expr: Expr {
                            instructions: vec![
                                Instruction::I32Const(8),
                            ]
                        },
                        init: "Hello, world!\n".bytes().collect(),
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
                                    // Store a pointer
                                    Instruction::I32Const(0),
                                    Instruction::I32Const(8),
                                    Instruction::I32Store(MemArg { align: 2, offset: 0, }),

                                    // Store the length
                                    Instruction::I32Const(4),
                                    Instruction::I32Const("Hello, world!\n".len() as i32),
                                    Instruction::I32Store(MemArg { align: 2, offset: 0, }),

                                    // Write
                                    Instruction::I32Const(1), // File descriptor
                                    Instruction::I32Const(0), // Data pointer
                                    Instruction::I32Const(1), // Number of strings
                                    Instruction::I32Const(0), // Where to put the number of bytes written
                                    Instruction::Call(0),

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

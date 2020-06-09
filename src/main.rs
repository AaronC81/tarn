mod wasm;
mod semantic_tree;
mod codegen;
mod parser;

use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
use std::sync::Arc;

use crate::wasm::{
    *,
    core::*,
    instruction::*,
    sections::{*, code_section::*, data_section::*, export_section::*, import_section::*, memory_section::*, type_section::*},
    module::*,
};
use crate::semantic_tree::{*, semanticize::*};
use crate::codegen::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parser::tarn_parser::program("
        import fn wasi_unstable fd_write(fd : Int, ptr : Int, len : Int, out : Int) -> Int;

        fn _start() -> Int {
            set! 0 8;
            set! 4 2;
            set! 8 65;
            set! 9 10;
            fd_write(1, 0, 1, 0)
        }
    ")?;

    let semantic = parsed.to_semantic_tree()?;

    let mut module = semantic.generate_module()?;
    module.export_sections[0].exports.push(Export {
        desc: ExportDesc::Func(1),
        name: "_start".into(),
    });

    let wasm = module.generate_wasm();

    let mut f = File::create("full.wasm")?;
    f.write_all(&wasm[..])?;

    return Ok(());

    let nodes = Node::Root(vec![
        Node::FunctionDeclaration(
            FuncId(0),
            Type::Function(
                vec![Type::Int, Type::Int, Type::Int, Type::Int],
                Some(Box::new(Type::Int))
            ),
            FunctionDefinition::Import("wasi_unstable".into(), "fd_write".into())
        ),
        Node::FunctionDeclaration(
            FuncId(1),
            Type::Function(vec![], Some(Box::new(Type::Int))),
            FunctionDefinition::Implementation(vec![], Box::new(Node::Block(
                vec![
                    // Set up array
                    Node::MemSet(Box::new(Node::IntegerConstant(0)), Box::new(Node::IntegerConstant(8))),
                    Node::MemSet(Box::new(Node::IntegerConstant(4)), Box::new(Node::IntegerConstant(2))),

                    // Set up string
                    Node::MemSet(Box::new(Node::IntegerConstant(8)), Box::new(Node::IntegerConstant(65))),
                    Node::MemSet(Box::new(Node::IntegerConstant(9)), Box::new(Node::IntegerConstant(10))),

                    // Call
                    Node::Call(FuncId(0), vec![
                        Node::IntegerConstant(1),
                        Node::IntegerConstant(0),
                        Node::IntegerConstant(1),
                        Node::IntegerConstant(0),
                    ])
                ],
                false
            )))
        )
    ]);

    //     Node(NodeType::Call(
    //     FuncId(0),
    //     vec![
    //         Box::new(Node(NodeType::IntegerConstant(1), ctx.clone())),
    //         Box::new(Node(NodeType::IntegerConstant(0), ctx.clone())),
    //         Box::new(Node(NodeType::IntegerConstant(1), ctx.clone())),
    //         Box::new(Node(NodeType::IntegerConstant(0), ctx.clone())),
    //     ],
    // ), ctx.clone());

    // let gen_global_ctx = Arc::new(CodeGenGlobalContext {
    //     function_table: [
    //         (FuncId(0), (
    //             Type::Function(
    //                 vec![Type::Int, Type::Int, Type::Int, Type::Int], Box::new(Type::Int)
    //             ),
    //             vec![],
    //         )),
    //     ].iter().cloned().collect(),
    //     type_table: [
    //         (
    //             TypeId(0),
    //             Type::Function(
    //                 vec![Type::Int, Type::Int, Type::Int, Type::Int], Box::new(Type::Int)
    //             ),
    //         ),
    //     ].iter().cloned().collect(),
    // });

    // let genctx = Arc::new(CodeGenContext {
    //     global: gen_global_ctx.clone(),
    //     parent: None,
    //     locals: vec![],
    // });

    let mut module = nodes.generate_module()?;
    module.export_sections[0].exports.push(Export {
        desc: ExportDesc::Func(1),
        name: "_start".into(),
    });

    let wasm = module.generate_wasm();

    let mut f = File::create("exp.wasm")?;
    f.write_all(&wasm[..])?;

    // println!("{:?}", code);

    return Ok(());

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

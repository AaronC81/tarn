use crate::semantic_tree::{Node, NodeType, Type};
use crate::wasm::instruction::Instruction;
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::{Display, Formatter};
use std::error::Error;

// A global context for code generation.
pub struct CodeGenGlobalContext {
    pub symbol_loc_table: HashMap<String, SymbolLoc>,

    // Maps types to their type IDs.
    pub type_table: Vec<Type>,
}

impl CodeGenGlobalContext {
    fn resolve(&self, name: String) -> Option<SymbolLoc> {
        self.symbol_loc_table.get(&name).cloned()
    }

    // Get the ID of a type, creating one if it doesn't exist.
    fn type_id(&mut self, t: Type) -> u32 {
        if let Some(pos) = self.type_table.iter().position(|x| x.clone() == t) {
            pos as u32
        } else {
            self.type_table.push(t);
            (self.type_table.len() - 1) as u32
        }
    }
}

// A per-function/block context for code generation.
pub struct CodeGenContext {
    pub global: Arc<CodeGenGlobalContext>,
    pub parent: Option<Arc<CodeGenContext>>,
    pub symbol_loc_table: HashMap<String, SymbolLoc>,
}

impl CodeGenContext {
    fn resolve(&self, name: String) -> Option<SymbolLoc> {
        if let Some(type_ref) = self.symbol_loc_table.get(&name) {
            Some(type_ref.clone())
        } else {
            if let Some(parent) = self.parent.clone() {
                parent.resolve(name)
            } else {
                self.global.resolve(name)
            }
        }
    }
}

#[derive(Clone)]
pub enum SymbolLoc {
    Local(u32),
    Func(u32),
}

#[derive(Debug, Clone)]
pub struct CodeGenError {
    reason: String,
}

impl CodeGenError {
    fn new(reason: String) -> CodeGenError {
        CodeGenError { reason }
    }
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "code gen error: {}", self.reason)
    }
}

impl Error for CodeGenError {}

pub trait CodeGen {
    fn generate_instructions(&self, ctx: Arc<CodeGenContext>) -> Result<Vec<Instruction>, CodeGenError>;
}

impl CodeGen for Node {
    fn generate_instructions(&self, ctx: Arc<CodeGenContext>) -> Result<Vec<Instruction>, CodeGenError> {
        use NodeType::*;
        use Instruction::*;
        match self {
            // TODO: all integer constants are i32 currently
            Node(IntegerConstant(i), _) => Ok(vec![ I32Const(*i as i32) ]),

            Node(Identifier(s), _) => match ctx.resolve(s.clone()) {
                Some(SymbolLoc::Local(i)) => Ok(vec![ LocalGet(i) ]),
                Some(SymbolLoc::Func(i)) => Ok(vec![ I32Const(i as i32) ]),
                None => Err(CodeGenError::new(format!("unable to resolve symbol {}", s))),
            },

            // TODO: make these more iterator
            // TODO: bad implementation, doesn't create a context or honor termination
            Node(NodeType::Block(ists, _), _) => {
                let mut result: Vec<Instruction> = vec![];
                for ist in ists {
                    let mut this = ist.generate_instructions(ctx.clone())?;
                    result.append(&mut this);
                }
                Ok(result)
            }

            Node(NodeType::StaticCall(target, args), _) => {
                let mut result: Vec<Instruction> = vec![];
                for arg in args {
                    let mut this = arg.generate_instructions(ctx.clone())?;
                    result.append(&mut this);
                }
                match ctx.resolve(target.clone()) {
                    Some(SymbolLoc::Func(id)) => {
                        result.push(Instruction::Call(id));
                        Ok(result)        
                    },
                    _ => Err(CodeGenError::new("unable to resolve function".into()))
                }
            }

            Node(NodeType::Call(target, args), _) => unimplemented!(),
            // {
            //     let mut result: Vec<Instruction> = vec![];
            //     for arg in args {
            //         let mut this = arg.generate_instructions(ctx.clone())?;
            //         result.append(&mut this);
            //     }
            //     let mut target = target.generate_instructions(ctx.clone())?;
            //     result.append(&mut target);
            //     // TODO: will be incorrect most of the time, this needs to take a type idx
            //     // Might have to limit this to static identifiers so we can look this up as part of a SymbolLoc for now
            //     // Add dynamic dispatch later
            //     result.push(Instruction::CallIndirect(0));
            //     Ok(result)
            // }
        }
    }
}
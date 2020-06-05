use crate::semantic_tree::{Node, NodeType, Type};
use crate::wasm::instruction::Instruction;
use crate::wasm::{LocalId, TypeId, FuncId};
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::{Display, Formatter};
use std::error::Error;

// A global context for code generation.
pub struct CodeGenGlobalContext {
    // A mapping of types to their WASM IDs.
    pub type_table: HashMap<Type, TypeId>,

    // A mapping of function IDs to their function type and local types.
    pub function_table: HashMap<FuncId, (Type, Vec<Type>)>,
}

// A per-function context for code generation.
pub struct CodeGenContext {
    pub global: Arc<CodeGenGlobalContext>,
    pub parent: Option<Arc<CodeGenContext>>,
    pub locals: HashMap<LocalId, Type>,
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

            Node(Local(LocalId(id)), _) => Ok(vec![ LocalGet(*id) ]),

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

            Node(NodeType::Call(FuncId(id), args), _) => {
                let mut result: Vec<Instruction> = vec![];
                for arg in args {
                    let mut this = arg.generate_instructions(ctx.clone())?;
                    result.append(&mut this);
                }
                result.push(Instruction::Call(*id));
                Ok(result)        
            },

            Node(NodeType::FunctionDefinition(_, _, _, _), _) =>
                Err(CodeGenError::new("can't generate instructions for a function definition".into())),
        }
    }
}
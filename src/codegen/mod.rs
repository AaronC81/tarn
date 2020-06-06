use crate::semantic_tree::{Node, Type};
use crate::wasm::{LocalId, TypeId, FuncId, module::Module, instruction::{Instruction, Expr}, sections::*, core::ValueType};
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::{Display, Formatter};
use std::error::Error;
use bimap::BiHashMap;
use code_section::Local;

// A global context for code generation.
pub struct CodeGenGlobalContext {
    // A mapping of type IDs to their types.
    pub type_table: BiHashMap<TypeId, Type>,

    // A mapping of function IDs to their function type and local types.
    pub function_table: HashMap<FuncId, (Type, Vec<Type>)>,
}

// A per-function context for code generation.
pub struct CodeGenContext {
    pub global: Arc<CodeGenGlobalContext>,
    pub parent: Option<Arc<CodeGenContext>>,
    pub locals: Vec<Type>,
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
    fn generate_module(&self) -> Result<Module, CodeGenError>;
    fn generate_instructions(&self, ctx: Arc<CodeGenContext>) -> Result<Vec<Instruction>, CodeGenError>;
}

impl CodeGen for Node {
    fn generate_module<'a>(&self) -> Result<Module, CodeGenError> {
        if let Node::Root(children) = self {
            // Create the function and type tables
            let mut type_table = BiHashMap::new();
            let mut function_table = Box::new(HashMap::new());

            // Iterate over all functions at the root
            for child in children {
                if let Node::FunctionDefinition(id, func_type, locals, body) = &**child {
                    // Create a function table entry
                    function_table.insert(*id, (func_type.clone(), locals.clone()));

                    // Create a type table entry, if there isn't one
                    if !type_table.contains_right(func_type) {
                        type_table.insert(TypeId(type_table.len() as u32), func_type.clone());
                    }
                } else {
                    return Err(CodeGenError::new("root must only contain valid function definitions".into()));
                }
            }

            // Create the global context
            let global_context = Arc::new(CodeGenGlobalContext {
                type_table: type_table.clone(), function_table: *function_table.clone()
            });

            // Create a code table
            let mut code_table: HashMap<FuncId, Vec<Instruction>> = HashMap::new();

            // Iterate over all functions at the root, again
            for child in children {
                if let Node::FunctionDefinition(id, _, locals, body) = &**child {
                    // Create a function context
                    let context = Arc::new(CodeGenContext {
                        global: global_context.clone(), parent: None, locals: locals.clone(),
                    });

                    code_table.insert(*id, body.generate_instructions(context)?);
                } else {
                    return Err(CodeGenError::new("root must only contain valid function definitions".into()));
                }
            }

            // TODO: validate keys are sequential

            // Ensure that the keys for both the function and code tables are the same
            if code_table.keys().len() != function_table.keys().len() || !code_table.keys().all(|x|  function_table.contains_key(x)) {
                return Err(CodeGenError::new("code and function table key mismatch".into()));
            }
            let mut function_ids: Vec<FuncId> = code_table.keys().cloned().collect();
            function_ids.sort();

            // Create the type section of the module
            let mut type_ids = type_table.left_values().collect::<Vec<_>>();
            type_ids.sort();
            let mut func_types: Vec<type_section::FuncType> = vec![];
            for id in type_ids {
                let t = type_table.get_by_left(id)
                    .ok_or(CodeGenError::new("missing type key".into()))?;

                    func_types.push(t.to_wasm_func_type()
                    .ok_or(CodeGenError::new("unable to convert function type".into()))?);
            }
            let type_section = TypeSection { func_types };

            // Create the code and function sections of the module
            let mut codes: Vec<code_section::Code> = vec![];
            let mut functions: Vec<u32> = vec![];
            for id in function_ids {
                let (func_type, locals) = function_table.get(&id)
                    .ok_or(CodeGenError::new("missing function key".into()))?;

                functions.push(type_table.get_by_right(func_type)
                    .ok_or(CodeGenError::new("no function type".into()))?.0);

                let code = code_table.get(&id)
                    .ok_or(CodeGenError::new("missing code key".into()))?;

                codes.push(code_section::Code {
                    func: code_section::Func {
                        locals: locals
                            .iter()
                            .map(|x| (*x).to_wasm_value_type())
                            .collect::<Option<Vec<ValueType>>>()
                            .ok_or(CodeGenError::new("unable to convert local type".into()))?
                            .iter()
                            .enumerate()
                            .map(|(n, t)| Local { n: n as u32, value_type: t.clone() } )
                            .collect(),
                        expr: Expr { instructions: (*code).clone() }
                    }
                })
            }
            let code_section = CodeSection { codes };
            let function_section = FunctionSection { types: functions };

            // Build a module
            Ok(Module {
                type_sections: vec![type_section],
                function_sections: vec![function_section],
                code_sections: vec![code_section],
                data_sections: vec![],
                memory_sections: vec![MemorySection {
                    memories: vec![
                        memory_section::Memory {
                            memory_type: crate::wasm::core::Limits {
                                min: 1,
                                max: None,
                            }
                        }
                    ]
                }],
                export_sections: vec![],
                import_sections: vec![],
            })
        } else {
            Err(CodeGenError::new("must generate module on a root node".into()))
        }
    }

    fn generate_instructions(&self, ctx: Arc<CodeGenContext>) -> Result<Vec<Instruction>, CodeGenError> {
        use Node::*;
        use Instruction::*;
        match self {
            // TODO: all integer constants are i32 currently
            IntegerConstant(i) => Ok(vec![ I32Const(*i as i32) ]),

            Local(LocalId(id)) => Ok(vec![ LocalGet(*id) ]),

            // TODO: make these more iterator
            // TODO: bad implementation, doesn't create a context or honor termination
            // TODO: drop unused values
            Node::Block(ists, _) => {
                let mut result: Vec<Instruction> = vec![];
                for ist in ists {
                    let mut this = ist.generate_instructions(ctx.clone())?;
                    result.append(&mut this);
                }
                Ok(result)
            }

            Node::Call(FuncId(id), args) => {
                let mut result: Vec<Instruction> = vec![];
                for arg in args {
                    let mut this = arg.generate_instructions(ctx.clone())?;
                    result.append(&mut this);
                }
                result.push(Instruction::Call(*id));
                Ok(result)
            },

            Node::FunctionDefinition(_, _, _, _) =>
                Err(CodeGenError::new("can't generate instructions for a function definition".into())),

            Node::Root(_) =>
                Err(CodeGenError::new("can't generate instructions for a root".into())),
        }
    }
}
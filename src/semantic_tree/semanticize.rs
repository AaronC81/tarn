use super::{Node as SemNode, Type, FunctionDefinition};
use crate::parser::{Node as ParseNode};
use crate::wasm::{LocalId, TypeId, FuncId};
use std::collections::HashMap;
use std::fmt::{Formatter, Display};
use std::sync::Arc;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct SemanticizeError {
    reason: String,
}

impl SemanticizeError {
    fn new<S: Into<String>>(reason: S) -> SemanticizeError {
        SemanticizeError { reason: reason.into() }
    }
}

impl Display for SemanticizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "code gen error: {}", self.reason)
    }
}

impl Error for SemanticizeError {}

pub trait Semanticize {
    fn to_semantic_tree(&self) -> Result<SemNode, SemanticizeError>;
}

impl ParseNode {
    // TODO: will need a context when user-defined types exist
    fn to_semantic_type(&self) -> Result<Type, SemanticizeError> {
        // TODO
        Ok(Type::Int)
    }

    // TODO: will need more fleshed-out context when locals exist
    fn to_semantic_node(&self, functions: Arc<HashMap<String, FuncId>>) -> Result<SemNode, SemanticizeError> {
        match self {
            // TODO: will need to look up locals
            ParseNode::Identifier(i) => unimplemented!(),
            ParseNode::Call(target, args) =>
                match &**target {
                    ParseNode::Identifier(s) => Ok(SemNode::Call(
                        functions
                            .get(s)
                            .ok_or(SemanticizeError::new(format!("no function {}", s)))?
                            .clone(),
                        args
                            .iter()
                            .map(|x| x.to_semantic_node(functions.clone()))
                            .collect::<Result<Vec<_>, _>>()?,
                    )),
                    _ => Err(SemanticizeError::new("must call an identifier")),
                },
            ParseNode::IntegerLiteral(i) => Ok(SemNode::IntegerConstant(*i)),
            ParseNode::Program(s) => Ok(SemNode::Root(s
                .iter()
                .map(|x| x.to_semantic_node(functions.clone()))
                .collect::<Result<Vec<_>, _>>()?)),
            ParseNode::FunctionImport { module, name, params, return_type } =>
                Ok(SemNode::FunctionDeclaration(
                    functions
                        .get(name)
                        .ok_or(SemanticizeError::new(format!("no internal function mapping for {}", name)))?
                        .clone(),
                    Type::Function(
                        params
                            .iter()
                            .map(|x| x.to_semantic_type())
                            .collect::<Result<Vec<_>, _>>()?,
                        Some(Box::new(return_type.to_semantic_type()?)),
                    ),
                    FunctionDefinition::Import(module.into(), name.into())
                )),
            ParseNode::FunctionImplementation { name, params, return_type, body } =>
                Ok(SemNode::FunctionDeclaration(
                    functions
                        .get(name)
                        .ok_or(SemanticizeError::new(format!("no internal function mapping for {}", name)))?
                        .clone(),
                    Type::Function(
                        params
                            .iter()
                            .map(|x| x.to_semantic_type())
                            .collect::<Result<Vec<_>, _>>()?,
                        Some(Box::new(return_type.to_semantic_type()?)),
                    ),
                    FunctionDefinition::Implementation(
                        vec![], // TODO when locals exist
                        Box::new(body.to_semantic_node(functions.clone())?),
                    )
                )),
            ParseNode::Block(body, term) => 
                Ok(SemNode::Block(
                    body
                        .iter()
                        .map(|x| x.to_semantic_node(functions.clone()))
                        .collect::<Result<Vec<_>, _>>()?,
                    *term,
                )),
            ParseNode::MemSet(target, value) =>
                Ok(SemNode::MemSet(
                    Box::new(target.to_semantic_node(functions.clone())?),
                    Box::new(value.to_semantic_node(functions.clone())?),
                )),
            _ => unimplemented!()
        }
    }
}

impl Semanticize for ParseNode {
    fn to_semantic_tree(&self) -> Result<SemNode, SemanticizeError> {
        let program_nodes = if let ParseNode::Program(nodes) = self {
            nodes
        } else {
            return Err(SemanticizeError::new("must convert a program"));
        };

        // Index functions, which can only be declared at the top level
        let mut functions: HashMap<String, FuncId> = HashMap::new();
        for node in program_nodes.iter() {
            match node {
                ParseNode::FunctionImplementation { name, .. } | ParseNode::FunctionImport { name, .. } =>
                    functions.insert(name.into(), FuncId(functions.len() as u32)),

                _ => return Err(SemanticizeError::new("must only have functions in program")),
            };
        }

        self.to_semantic_node(Arc::new(functions))
    }
}

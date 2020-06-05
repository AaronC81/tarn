use std::collections::HashMap;
use std::sync::Arc;
use crate::wasm::{LocalId, TypeId, FuncId};

pub struct Node(pub NodeType, pub Arc<TreeContext>);

pub struct TreeContext {
    pub parent: Option<Arc<TreeContext>>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Function(Vec<Type>, Box<Type>),   
}

pub enum NodeType {
    FunctionDefinition(FuncId, Vec<Type>, Type, Vec<Type>), // ID, params, returns, locals
    IntegerConstant(i64),
    Local(LocalId),
    Call(FuncId, Vec<Box<Node>>),
    Block(Vec<Box<Node>>, bool), // bool = is this block terminated?
}
use std::collections::HashMap;
use std::sync::Arc;
use crate::wasm::{LocalId, TypeId, FuncId, sections::type_section::FuncType, core::ValueType};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Function(Vec<Type>, Option<Box<Type>>),   
}

impl Type {
    pub fn to_wasm_func_type(&self) -> Option<FuncType> {
        match self {
            Type::Function(params, ret) =>
                Some(FuncType {
                    parameters: params.iter().map(|x| x.to_wasm_value_type()).collect::<Option<Vec<_>>>()?,
                    results: if let Some(r) = ret { vec![(**r).clone().to_wasm_value_type()?] } else { vec![] },
                }),
            _ => None,
        }   
    }

    pub fn to_wasm_value_type(&self) -> Option<ValueType> {
        match self {
            Type::Int => Some(ValueType::I32),
            _ => None,
        }
    }
}

pub enum Node {
    Root(Vec<Box<Node>>),
    FunctionDefinition(FuncId, Type, Vec<Type>, Box<Node>), // ID, type, locals, body
    IntegerConstant(i64),
    Local(LocalId),
    Call(FuncId, Vec<Box<Node>>),
    Block(Vec<Node>, bool), // bool = is this block terminated?
}
use std::collections::HashMap;
use std::sync::Arc;

pub struct Node(pub NodeType, pub Arc<TreeContext>);

pub struct TreeContext {
    pub parent: Option<Arc<TreeContext>>,
    pub symbol_type_table: HashMap<String, Type>,
}

impl TreeContext {
    fn resolve(&self, name: String) -> Option<Type> {
        if let Some(type_ref) = self.symbol_type_table.get(&name) {
            Some(type_ref.clone())
        } else {
            if let Some(parent) = self.parent.clone() {
                parent.resolve(name)
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Function(Vec<Box<Type>>, Box<Type>),   
}

pub enum NodeType {
    IntegerConstant(i64),
    Identifier(String),
    Call(Box<Node>, Vec<Box<Node>>),
    StaticCall(String, Vec<Box<Node>>),
    Block(Vec<Box<Node>>, bool), // bool = is this block terminated?
}
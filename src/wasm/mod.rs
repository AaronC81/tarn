pub mod core;
pub mod instruction;
pub mod module;
pub mod sections;

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct FuncId(pub u32);
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct LocalId(pub u32);
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TypeId(pub u32);
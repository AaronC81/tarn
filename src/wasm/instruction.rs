use crate::wasm::core::{WasmCodeGen, ValueType, encode_i32, encode_u32};

#[derive(Debug, Clone)]
pub enum Instruction {
    Unreachable,
    Nop,
    Block(BlockType, Vec<Instruction>), Loop(BlockType, Vec<Instruction>),
    If(BlockType, Vec<Instruction>), IfElse(BlockType, Vec<Instruction>, Vec<Instruction>),
    Branch(u32), BranchIf(u32),
    // TODO: BranchTable,
    Return, Call(u32), CallIndirect(u32),
    Drop, Select,
    LocalGet(u32), LocalSet(u32), LocalTee(u32),
    GlobalGet(u32), GlobalSet(u32),
    I32Load(MemArg), I64Load(MemArg), F32Load(MemArg), F64Load(MemArg),
    I32Load8S(MemArg), I32Load8U(MemArg), I32Load16S(MemArg), I32Load16U(MemArg),
    I64Load8S(MemArg), I64Load8U(MemArg), I64Load16S(MemArg), I64Load16U(MemArg), I64Load32S(MemArg), I64Load32U(MemArg),
    I32Store(MemArg), I64Store(MemArg), F32Store(MemArg), F64Store(MemArg),
    I32Store8(MemArg), I32Store16(MemArg),
    I64Store8(MemArg), I64Store16(MemArg), I64Store32(MemArg),
    MemorySize, MemoryGrow,
    I32Const(i32), I64Const(i64),
    // TODO: unsure if these float types are correct
    F32Const(f32), F64Const(f64),

    // TODO the remaining instructions
}

impl Instruction {
    fn opcode(&self) -> u8 {
        use Instruction::*;

        match self {
            Unreachable => 0x00,
            Nop => 0x01,
            Block(_, _) => 0x02, Loop(_, _) => 0x03,
            If(_, _) => 0x04, IfElse(_, _, _) => 0x04,
            Branch(_) => 0x0C, BranchIf(_) => 0x0D,
            Return => 0x0F, Call(_) => 0x10, CallIndirect(_) => 0x11,
            Drop => 0x1A, Select => 0x1B,
            LocalGet(_) => 0x20, LocalSet(_) => 0x21, LocalTee(_) => 0x22,
            GlobalGet(_) => 0x23, GlobalSet(_) => 0x24,
            I32Load(_) => 0x28, I64Load(_) => 0x29, F32Load(_) => 0x2A, F64Load(_) => 0x2B,
            I32Load8S(_) => 0x2C, I32Load8U(_) => 0x2D, I32Load16S(_) => 0x2E, I32Load16U(_) => 0x2F,
            I64Load8S(_) => 0x30, I64Load8U(_) => 0x31, I64Load16S(_) => 0x32, I64Load16U(_) => 0x33, I64Load32S(_) => 0x34, I64Load32U(_) => 0x35,
            I32Store(_) => 0x36, I64Store(_) => 0x37, F32Store(_) => 0x38, F64Store(_) => 0x39,
            I32Store8(_) => 0x3A, I32Store16(_) => 0x3B,
            I64Store8(_) => 0x3C, I64Store16(_) => 0x3D, I64Store32(_) => 0x3E,
            MemorySize => 0x3F, MemoryGrow => 0x40,
            I32Const(_) => 0x41, I64Const(_) => 0x42,
            F32Const(_) => 0x43, F64Const(_) => 0x44,
        }
    }

    fn operands(&self) -> Vec<u8> {
        use Instruction::*;

        match self {
            Block(t, i) | Loop(t, i) | If(t, i) => [
                t.generate_wasm(),
                self.generate_wasm_seq(i),
                vec![0x0B]].concat(),

            IfElse(t, i1, i2) => [
                t.generate_wasm(),
                self.generate_wasm_seq(i1),
                vec![0x05],
                self.generate_wasm_seq(i2),
                vec![0x0B]].concat(),

            Branch(x) | BranchIf(x) | Call(x) | LocalGet(x) | LocalSet(x) | LocalTee(x) | GlobalGet(x) | GlobalSet(x) => encode_u32(*x),

            CallIndirect(x) => [encode_u32(*x), vec![0x00]].concat(),
 
            I32Load(m) | I64Load(m) | F32Load(m) | F64Load(m) | I32Load8S(m) | I32Load8U(m) | I32Load16S(m) | I32Load16U(m)
                | I64Load8S(m) | I64Load8U(m) | I64Load16S(m) | I64Load16U(m) | I64Load32S(m) | I64Load32U(m) 
                | I32Store(m) | I64Store(m) | F32Store(m) | F64Store(m) | I32Store8(m) | I32Store16(m) | I64Store8(m)
                | I64Store16(m) | I64Store32(m) => m.generate_wasm(),
            
            I32Const(x) => encode_i32(*x),
            
            // TODO
            I64Const(_) | F32Const(_) | F64Const(_) => unimplemented!(),

            Unreachable | Nop | MemorySize | MemoryGrow | Drop | Select | Return => vec![],
        }
    }
}

impl WasmCodeGen for Instruction {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            vec![self.opcode()],
            self.operands(), 
        ].concat()
    }
}

#[derive(Debug, Clone)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
    TypeIndex(u32), // TODO this is hyper cursed or something
}

impl WasmCodeGen for BlockType {
    fn generate_wasm(&self) -> Vec<u8> {
        match self {
            BlockType::Empty => vec![0x40],
            BlockType::ValueType(t) => t.generate_wasm(),
            BlockType::TypeIndex(i) => i.generate_wasm(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl WasmCodeGen for MemArg {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.align.generate_wasm(),
            self.offset.generate_wasm(),
        ].concat()
    }
}

pub struct Expr {
    pub instructions: Vec<Instruction>,
}

impl WasmCodeGen for Expr {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.generate_wasm_seq(&self.instructions),
            vec![0x0B],
        ].concat()
    }
}


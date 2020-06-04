use leb128;

pub fn encode_u32(n: u32) -> Vec<u8> {
    let mut buf = vec![];
    leb128::write::unsigned(&mut buf, n as u64).expect("leb128 conversion failed");
    buf
}

pub fn encode_i32(n: i32) -> Vec<u8> {
    let mut buf = vec![];
    leb128::write::signed(&mut buf, n as i64).expect("leb128 conversion failed");
    buf
}


pub trait WasmCodeGen {
    fn generate_wasm(&self) -> Vec<u8>;

    fn generate_wasm_seq<T: WasmCodeGen>(&self, items: &Vec<T>) -> Vec<u8> {
        items.iter().map(|x| x.generate_wasm()).flatten().collect()
    }

    fn generate_wasm_vec<T: WasmCodeGen>(&self, items: &Vec<T>) -> Vec<u8> {
        [
            encode_u32(items.len() as u32),
            self.generate_wasm_seq(items),
        ].concat()
    }
}

impl WasmCodeGen for String {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            encode_u32(self.len() as u32),
            self.bytes().collect(),
        ].concat()
    }
}

impl WasmCodeGen for u32 {
    fn generate_wasm(&self) -> Vec<u8> {
        encode_u32(*self)
    }
}

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

pub struct MemArg {
    align: u32,
    offset: u32,
}

impl WasmCodeGen for MemArg {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.align.generate_wasm(),
            self.offset.generate_wasm(),
        ].concat()
    }
}

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

pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    fn to_byte(&self) -> u8 {
        match self {
            ValueType::I32 => 0x7F,
            ValueType::I64 => 0x7E,
            ValueType::F32 => 0x7D,
            ValueType::F64 => 0x7C,
        }
    }
}

impl WasmCodeGen for ValueType {
    fn generate_wasm(&self) -> Vec<u8> {
        vec![self.to_byte()]
    }
}

pub struct Module {
    pub type_sections: Vec<TypeSection>,
    pub import_sections: Vec<ImportSection>,
    pub function_sections: Vec<FunctionSection>,
    pub memory_sections: Vec<MemorySection>,
    pub export_sections: Vec<ExportSection>,
    pub code_sections: Vec<CodeSection>,
}

impl Module {
    fn magic(&self) -> Vec<u8> {
        vec![0x00, 0x61, 0x73, 0x6D]
    }

    fn version(&self) -> Vec<u8> {
        vec![0x01, 0x00, 0x00, 0x00]
    }
}

impl WasmCodeGen for Module {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.magic(),
            self.version(),
            self.generate_wasm_seq(&self.type_sections),
            self.generate_wasm_seq(&self.import_sections),
            self.generate_wasm_seq(&self.function_sections),
            self.generate_wasm_seq(&self.memory_sections),
            self.generate_wasm_seq(&self.export_sections),
            self.generate_wasm_seq(&self.code_sections),
        ].concat()
    }
}

pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl WasmCodeGen for Limits {
    fn generate_wasm(&self) -> Vec<u8> {
        if let Some(max) = self.max {
            [
                vec![0x01],
                encode_u32(self.min),
                encode_u32(max),
            ].concat()
        } else {
            [
                vec![0x00],
                encode_u32(self.min),
            ].concat()
        }
    }
}

pub trait Section {
    const ID: u8;
}

pub trait BodySection {
    const ID: u8;
    type BodyItem : WasmCodeGen;

    fn body_item(&self) -> &Vec<Self::BodyItem>;
}

impl<T : BodySection> Section for T {
    const ID: u8 = Self::ID;
}

impl<T : BodySection> WasmCodeGen for T {
    fn generate_wasm(&self) -> Vec<u8> {
        let body = self.generate_wasm_vec(&self.body_item());
        [
            vec![Self::ID],
            encode_u32(body.len() as u32),
            body,
        ].concat()
    }
}

pub struct MemorySection {
    pub memories: Vec<Memory>,
}

impl BodySection for MemorySection {
    const ID: u8 = 5;
    type BodyItem = Memory;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.memories }
}

pub struct Memory {
    pub memory_type: Limits,
}

impl WasmCodeGen for Memory {
    fn generate_wasm(&self) -> Vec<u8> {
        self.memory_type.generate_wasm()
    }
}

pub struct ImportSection {
    pub imports: Vec<Import>,
}

impl BodySection for ImportSection {
    const ID: u8 = 2;
    type BodyItem = Import;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.imports }
}

pub struct Import {
    pub module: String,
    pub name: String,
    pub desc: ImportDesc,
}

impl WasmCodeGen for Import {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.module.generate_wasm(),
            self.name.generate_wasm(),
            self.desc.generate_wasm(),
        ].concat()
    }
}

pub enum ImportDesc {
    // TODO: other import types
    Func(u32),
}

impl WasmCodeGen for ImportDesc {
    fn generate_wasm(&self) -> Vec<u8> {
        match self {
            ImportDesc::Func(i) => [
                vec![0x00],
                encode_u32(*i),
            ].concat()
        }
    }
}

pub struct FunctionSection {
    pub types: Vec<u32>,
}

impl BodySection for FunctionSection {
    const ID: u8 = 3;
    type BodyItem = u32;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.types }
}

pub struct TypeSection {
    pub func_types: Vec<FuncType>,
}

impl BodySection for TypeSection {
    const ID: u8 = 1;
    type BodyItem = FuncType;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.func_types }
}

pub struct FuncType {
    pub parameters: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl WasmCodeGen for FuncType {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            vec![0x60],
            self.generate_wasm_vec(&self.parameters),
            self.generate_wasm_vec(&self.results),
        ].concat()
    }
}

pub struct CodeSection {
    pub codes: Vec<Code>,
}

impl BodySection for CodeSection {
    const ID: u8 = 10;
    type BodyItem = Code;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.codes }
}
pub struct ExportSection {
    pub exports: Vec<Export>,
}

impl BodySection for ExportSection {
    const ID: u8 = 7;
    type BodyItem = Export;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.exports }
}

pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

impl WasmCodeGen for Export {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.name.generate_wasm(),
            self.desc.generate_wasm(),
        ].concat()
    }
}

pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Mem(u32),
    Global(u32),
}

impl ExportDesc {
    fn type_byte(&self) -> u8 {
        match self {
            ExportDesc::Func(_) => 0,
            ExportDesc::Table(_) => 1,
            ExportDesc::Mem(_) => 2,
            ExportDesc::Global(_) => 3,
        }
    }
}


impl WasmCodeGen for ExportDesc {
    fn generate_wasm(&self) -> Vec<u8> {

        match self {
            ExportDesc::Func(i) |
            ExportDesc::Table(i) |
            ExportDesc::Mem(i) |
            ExportDesc::Global(i) => [
                vec![self.type_byte()],
                encode_u32(*i),
            ].concat()
        }
    }
}

pub struct Code {
    pub func: Func,
}

impl WasmCodeGen for Code {
    fn generate_wasm(&self) -> Vec<u8> {
        let func = self.func.generate_wasm();
        [
            encode_u32(func.len() as u32),
            func,
        ].concat()
    }
}

pub struct Func {
    pub locals: Vec<Local>,
    pub expr: Expr,
}

impl WasmCodeGen for Func {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            self.generate_wasm_vec(&self.locals),
            self.expr.generate_wasm(),
        ].concat()
    }
}

pub struct Local {
    pub n: u32,
    pub value_type: ValueType,
}

impl WasmCodeGen for Local {
    fn generate_wasm(&self) -> Vec<u8> {
        [
            encode_u32(self.n),
            vec![self.value_type.to_byte()],
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

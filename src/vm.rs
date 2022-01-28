#[derive(Debug, Clone, Copy)]
pub(crate) enum Inst {
    Add,
    Call(u8),
    Class(u8),
    CloseUpVal,
    Closure(u8),
    Const(u8),
    DefGlobal(u8),
    Div,
    Equal,
    False,
    GetGlobal(u8),
    GetLocal(u8),
    GetProp(u8),
    GetSuper(u8),
    GetUpVal(u8),
    Greater,
    Inherit,
    Invoke(u8, u8),
    JumpIf(u16),
    JumpUnless(u16),
    Less,
    Loop(u16),
    Method(u8),
    Mul,
    Neg,
    Nil,
    Not,
    Pop,
    Print,
    Return,
    SetGlobal(u8),
    SetLocal(u8),
    SetProp(u8),
    SetUpVal(u8),
    Sub,
    SuperInvoke(u8, u8),
    True,
}

#[derive(Debug)]
pub(crate) enum Val {}

#[derive(Debug, Default)]
pub(crate) struct Chunk {
    /// A collection of all [`OpCode`]s and constant pool indexes in this
    /// [`Chunk`].
    code: Vec<Inst>,
    /// The constant pool.
    consts: Vec<Val>,
    /// Line numbers (of those [`OpCode`]s), for runtime error reporting.
    lines: Vec<usize>,
}

impl Chunk {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Pushes a new instruction ([`Inst`]) into the chunk.
    /// Returns the index of the last instruction.
    pub(crate) fn push(&mut self, instruction: Inst, line: usize) -> usize {
        self.code.push(instruction);
        self.lines.push(line);
        self.code.len() - 1
    }

    /// Pushes a new constant into the chunk.
    /// Returns the index of the last constant.
    pub(crate) fn push_const(&mut self, const_: Val) -> usize {
        self.consts.push(const_);
        self.consts.len() - 1
    }
}

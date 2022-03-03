use rle_vec::RleVec;

use super::{Inst, Val};

#[derive(Debug, Default)]
pub(crate) struct Chunk {
    /// A collection of all [`OpCode`]s and constant pool indexes in this
    /// [`Chunk`].
    pub(crate) code: Vec<Inst>,
    /// The constant pool.
    pub(crate) consts: Vec<Val>,
    /// Line numbers (of those [`OpCode`]s), for runtime error reporting.
    pub(crate) lines: RleVec<usize>,
}

impl Chunk {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get(&self, idx: usize) -> Option<&Inst> {
        self.code.get(idx)
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
    pub(crate) fn push_const(&mut self, konst: Val) -> usize {
        self.consts.push(konst);
        self.consts.len() - 1
    }
}

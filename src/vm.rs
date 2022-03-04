use std::{ops::Add, sync::Arc};

use anyhow::{Context, Result};

pub(crate) use self::{chunk::Chunk, inst::Inst, val::Val};

mod chunk;
mod inst;
mod val;

/// The Lox Bytecode Virtual Machine.
#[derive(Debug, Default, Clone)]
pub(crate) struct Vm {
    chunk: Arc<Chunk>,
    /// Program counter.
    pc: usize,
    stack: Vec<Val>,
}

impl Vm {
    fn new(chunk: &Arc<Chunk>) -> Self {
        Self {
            chunk: Arc::clone(chunk),
            ..Self::default()
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let inst = self.chunk.get(self.pc);
            if inst.is_none() {
                break;
            }
            match *inst.unwrap() {
                Inst::Add => {
                    let stack_underflow = "vm error: stack underflow";
                    let rhs = self.stack.pop().context(stack_underflow)?;
                    let lhs = self.stack.pop().context(stack_underflow)?;
                    self.stack.push((lhs + rhs)?);
                }
                Inst::Call(_) => todo!(),
                Inst::Class(_) => todo!(),
                Inst::CloseUpVal => todo!(),
                Inst::Closure(_) => todo!(),
                Inst::Const(const_idx) => self
                    .stack
                    .push(self.chunk.consts[const_idx as usize].clone()),
                Inst::DefGlobal(_) => todo!(),
                Inst::Div => todo!(),
                Inst::Equal => todo!(),
                Inst::False => todo!(),
                Inst::GetGlobal(_) => todo!(),
                Inst::GetLocal(_) => todo!(),
                Inst::GetProp(_) => todo!(),
                Inst::GetSuper(_) => todo!(),
                Inst::GetUpVal(_) => todo!(),
                Inst::Greater => todo!(),
                Inst::Inherit => todo!(),
                Inst::Invoke(_, _) => todo!(),
                Inst::JumpIf(_) => todo!(),
                Inst::JumpUnless(_) => todo!(),
                Inst::Less => todo!(),
                Inst::Loop(_) => todo!(),
                Inst::Method(_) => todo!(),
                Inst::Mul => todo!(),
                Inst::Neg => todo!(),
                Inst::Nil => todo!(),
                Inst::Not => todo!(),
                Inst::Pop => todo!(),
                Inst::Print => todo!(),
                Inst::Return => break,
                Inst::SetGlobal(_) => todo!(),
                Inst::SetLocal(_) => todo!(),
                Inst::SetProp(_) => todo!(),
                Inst::SetUpVal(_) => todo!(),
                Inst::Sub => todo!(),
                Inst::SuperInvoke(_, _) => todo!(),
                Inst::True => todo!(),
            }
            self.pc += 1;
        }
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use assert_matches::assert_matches;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_works() {
        use Val::*;
        let mut chunk = Chunk::new();
        let k1 = chunk.push_const(Num(11.4));
        let k2 = chunk.push_const(Num(5.14));
        chunk.push(Inst::Const(k1 as _), 42);
        chunk.push(Inst::Const(k2 as _), 42);
        chunk.push(Inst::Add, 43);
        chunk.push(Inst::Return, 44);
        let chunk = Arc::new(chunk);

        let mut vm = Vm::new(&chunk);
        vm.run().unwrap();
        assert_matches!(vm.stack.pop(), Some(Num(x)) if x == 16.54);
    }
}

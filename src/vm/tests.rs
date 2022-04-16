#![cfg(test)]

use assert_matches::assert_matches;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn it_works() {
    use Val::*;
    let mut chunk = Chunk::new();
    let k1 = chunk.push_const(Num(11.4));
    let k2 = chunk.push_const(Num(5.14));

    [
        (Inst::Const(k1 as _), 42),
        (Inst::Const(k2 as _), 42),
        (Inst::Add, 43),
        (Inst::Return, 44),
    ]
    .into_iter()
    .for_each(|(inst, ln)| {
        chunk.push(inst, ln);
    });

    let chunk = Arc::new(chunk);
    let mut vm = Vm::new(&chunk);
    vm.run().unwrap();
    assert_matches!(vm.stack.pop(), Some(Num(x)) if x == 16.54);
}

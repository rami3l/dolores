#![allow(clippy::module_name_repetitions)]

use std::{
    fmt::{self, Debug, Display},
    iter::Peekable,
};

use thiserror::Error;

/*
 * Terminology:
 * token precedence	= left binding power, lbp
 * subexpression precedence	= right binding power, rbp
 * head handler function = null denotation, nud
 * tail handler function = left denotation, led
 * src: <https://abarker.github.io/typped/pratt_parsing_intro.html>
 *
 *         <lbp>  <rbp>  <nbp> <kind>
 * Nilfix:  MIN |  MIN |  MAX | nud
 * Prefix:  MIN |   bp |  MAX | nud
 * Postfix:  bp |  MIN |  MAX | led
 * InfixL:   bp |   bp | bp+1 | led
 * InfixR:   bp | bp-1 | bp+1 | led
 * InfixN:   bp |*bp+1*|   bp | led
 * src: <https://github.com/segeljakt/pratt/blob/master/src/lib.rs>
 */

/// Operator associativity.
#[derive(Copy, Clone)]
pub enum Assoc {
    /// Left associativity.
    L,
    /// Right associativity.
    R,
    /// No associativity, i.e. this operator cannot be chained.
    NA,
}

/// Operator precedence.
///
/// # Note
/// Ideally, an operator's precedence should be a multiple of 10 when created,
/// so please use the [`Prec::new_10x`] method to construct new precedences.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Prec(pub u32);

impl Prec {
    pub fn new_10x(prec: u32) -> Self {
        Self(10 * prec)
    }

    const MIN: Prec = Prec(u32::MIN);
    const MAX: Prec = Prec(u32::MAX);

    const fn raise(mut self) -> Prec {
        self.0 += 1;
        self
    }

    const fn lower(mut self) -> Prec {
        self.0 -= 1;
        self
    }
}

#[derive(Copy, Clone)]
pub enum Affix {
    Nilfix,
    Infix(Prec, Assoc),
    Prefix(Prec),
    Postfix(Prec),
}

#[derive(Error, Debug)]
pub enum PrattError<I, E> {
    UserError(E),
    EmptyInput,
    UnexpectedNilfix(I),
    UnexpectedPrefix(I),
    UnexpectedInfix(I),
    UnexpectedPostfix(I),
}

impl<I: Debug, E: Display> Display for PrattError<I, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrattError::UserError(e) => write!(f, "{}", e),
            PrattError::EmptyInput => write!(f, "Pratt parser was called with empty input."),
            PrattError::UnexpectedNilfix(t) => {
                write!(f, "Expected Infix or Postfix, found Nilfix {:?}", t)
            }
            PrattError::UnexpectedPrefix(t) => {
                write!(f, "Expected Infix or Postfix, found Prefix {:?}", t)
            }
            PrattError::UnexpectedInfix(t) => {
                write!(f, "Expected Nilfix or Prefix, found Infix {:?}", t)
            }
            PrattError::UnexpectedPostfix(t) => {
                write!(f, "Expected Nilfix or Prefix, found Postfix {:?}", t)
            }
        }
    }
}

pub trait PrattParser<Inputs: Iterator<Item = Self::Input>> {
    type Error;
    type Input;
    type Output: Sized;

    /// The affix info of an operator, possibly with its precedence and
    /// association.
    fn op_info(&mut self, input: &Self::Input) -> Result<Affix, Self::Error>;

    /// The primary expression handler.
    fn primary(&mut self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// The infix expression builder.
    fn infix(
        &mut self,
        lhs: Self::Output,
        op: Self::Input,
        rhs: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// The prefix expression builder.
    fn prefix(&mut self, op: Self::Input, rhs: Self::Output) -> Result<Self::Output, Self::Error>;

    /// The postfix expression builder.
    fn postfix(&mut self, lhs: Self::Output, op: Self::Input) -> Result<Self::Output, Self::Error>;

    /// The default entry point of this parser, starting with a right binding
    /// power of `0`.
    fn parse_0(
        &mut self,
        inputs: &mut Peekable<&mut Inputs>,
    ) -> Result<Self::Output, PrattError<Self::Input, Self::Error>> {
        self.parse(inputs, Prec(0))
    }

    fn parse(
        &mut self,
        tail: &mut Peekable<&mut Inputs>,
        right_pow: Prec,
    ) -> Result<Self::Output, PrattError<Self::Input, Self::Error>> {
        if let Some(head) = tail.next() {
            let info = self.op_info(&head).map_err(PrattError::UserError)?;
            let mut next_pow = self.next_pow(info);
            let mut node = self.null_deno(head, tail, info);
            while let Some(head) = tail.peek() {
                let info = self.op_info(head).map_err(PrattError::UserError)?;
                let left_pow = self.left_pow(info);
                // We continue to consume new tokens when:
                // - The current subexpression has a lower precedence (`right_pow`) than that of
                //   the next operator (`left_pow`)
                // - The next operator is permitted by the rightmost token of the current
                //   subexpression in that its precedence (`left_pow`) is lower than the maximal
                //   precedence it can take (`next_pow`).
                if right_pow < left_pow && left_pow < next_pow {
                    let head = tail.next().unwrap();
                    next_pow = self.next_pow(info);
                    node = self.left_deno(head, tail, info, node?);
                } else {
                    break;
                }
            }
            node
        } else {
            Err(PrattError::EmptyInput)
        }
    }

    /// Null denotation
    fn null_deno(
        &mut self,
        head: Self::Input,
        tail: &mut Peekable<&mut Inputs>,
        info: Affix,
    ) -> Result<Self::Output, PrattError<Self::Input, Self::Error>> {
        match info {
            Affix::Prefix(prec) => {
                let rhs = self.parse(tail, prec.lower());
                self.prefix(head, rhs?).map_err(PrattError::UserError)
            }
            Affix::Nilfix => self.primary(head).map_err(PrattError::UserError),
            Affix::Postfix(_) => Err(PrattError::UnexpectedPostfix(head)),
            Affix::Infix(_, _) => Err(PrattError::UnexpectedInfix(head)),
        }
    }

    /// Left denotation
    fn left_deno(
        &mut self,
        head: Self::Input,
        tail: &mut Peekable<&mut Inputs>,
        info: Affix,
        lhs: Self::Output,
    ) -> Result<Self::Output, PrattError<Self::Input, Self::Error>> {
        match info {
            Affix::Infix(prec, assoc) => {
                let rhs = self.parse(
                    tail,
                    match assoc {
                        Assoc::L => prec,
                        Assoc::R => prec.lower(),
                        Assoc::NA => prec.raise(),
                    },
                );
                self.infix(lhs, head, rhs?).map_err(PrattError::UserError)
            }
            Affix::Postfix(_) => self.postfix(lhs, head).map_err(PrattError::UserError),
            Affix::Nilfix => Err(PrattError::UnexpectedNilfix(head)),
            Affix::Prefix(_) => Err(PrattError::UnexpectedPrefix(head)),
        }
    }

    /// The "left binding power" is the precedence of a token.
    fn left_pow(&mut self, info: Affix) -> Prec {
        use Affix::*;
        match info {
            Nilfix | Prefix(_) => Prec::MIN,
            Infix(prec, _) | Postfix(prec) => prec,
        }
    }

    /// The "next binding power" is the maximal precedence of the upcoming token
    /// as permitted by the current one.
    fn next_pow(&mut self, info: Affix) -> Prec {
        use Affix::*;
        match info {
            Nilfix | Prefix(_) | Postfix(_) => Prec::MAX,
            Infix(prec, Assoc::L | Assoc::R) => prec.raise(),
            Infix(prec, Assoc::NA) => prec,
        }
    }
}

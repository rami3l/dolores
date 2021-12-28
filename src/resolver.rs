mod expr;
mod stmt;

use std::collections::HashMap;

use anyhow::Result;

use crate::{interpreter::Interpreter, lexer::Token, parser::Stmt};

#[derive(Debug, Clone, Default)]
pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<Scope>,
    function_ctx: Option<FunctionContext>,
    class_ctx: Option<ClassContext>,
}

// See: <https://www.craftinginterpreters.com/resolving-and-binding.html#resolving-variable-declarations>
#[derive(Debug, Clone, Copy)]
pub enum ResolutionState {
    /// The variable is added to the innermost scope, so that it shadows any
    /// outer one, and so that we know its existence.
    Declared,
    /// The variable is fully resolved by the resolver and ready to be used.
    Defined,
}

pub type Scope = HashMap<String, ResolutionState>;

#[derive(Debug, Clone, Copy)]
pub enum FunctionContext {
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, Copy)]
pub enum ClassContext {
    Class,
    Subclass,
}

impl Resolver {
    #[must_use]
    pub fn new(interpreter: Interpreter) -> Self {
        Resolver {
            interpreter,
            ..Resolver::default()
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    /// Sets the resolution state of the given `token` in the currently smallest
    /// scope, returning the last state if exists.
    fn set_state(&mut self, token: &Token, state: ResolutionState) -> Option<ResolutionState> {
        self.scopes
            .last_mut()
            .and_then(|last| last.insert(token.lexeme.clone(), state))
    }

    fn declare(&mut self, token: &Token) -> Option<ResolutionState> {
        self.set_state(token, ResolutionState::Declared)
    }

    fn define(&mut self, token: &Token) -> Option<ResolutionState> {
        self.set_state(token, ResolutionState::Defined)
    }

    fn resolve_local(&mut self, name: &Token) {
        self.scopes
            .iter()
            .rev()
            .enumerate()
            .find_map(|(distance, scope)| {
                scope
                    .contains_key(&name.lexeme)
                    .then(|| self.interpreter.locals.insert(name.clone(), distance))
            });
    }

    pub(crate) fn resolve_lambda(
        &mut self,
        ctx: FunctionContext,
        params: &[Token],
        body: Vec<Stmt>,
    ) -> Result<()> {
        let old_ctx = self.function_ctx.replace(ctx);
        self.begin_scope();
        for it in params {
            self.declare(it);
            self.define(it);
        }
        body.into_iter().try_for_each(|it| self.resolve_stmt(it))?;
        self.end_scope();
        self.function_ctx = old_ctx;
        Ok(())
    }

    pub(crate) fn resolve(mut self, stmts: impl IntoIterator<Item = Stmt>) -> Result<Interpreter> {
        stmts.into_iter().try_for_each(|it| self.resolve_stmt(it))?;
        Ok(self.interpreter)
    }
}

use crate::lexer::Token::{self, *};

enum Expr {
    Assign {
        name: Token,
        val: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Get {
        obj: Box<Expr>,
        name: Token,
    },
    Grouping(Box<Expr>),
    Literal(Lit),
    Logical {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Set {
        obj: Box<Expr>,
        name: Token,
        to: Box<Expr>,
    },
    Super {
        kw: Token,
        method: Token,
    },
    This(Token),
    Unary {
        op: Token,
        rhs: Box<Expr>,
    },
    Variable(Token),
}

enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Token,
        /// # Note
        /// Tokenhis value **must** be of variant `Expr::Variable`.
        superclass: Expr,
        /// # Note
        /// Tokenhis `Vec` **must** contain instances of `Stmt::Function`.
        methods: Vec<Stmt>,
    },
    Expression(Expr),
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Box<Stmt>,
    },
    Print(Expr),
    Return {
        kw: Token,
        val: Expr,
    },
    Var {
        name: Token,
        init: Expr,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
}

enum Lit {
    Bool(bool),
    Number(f64),
    Nil,
}

struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, idx: 0 }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.idx).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let res = self.peek()?;
        self.idx += 1;
        Some(res)
    }

    fn previous(&self) -> Option<Token> {
        self.tokens.get(self.idx - 1).cloned()
    }

    fn advance_if_matches(&mut self, patterns: &[Token]) -> bool {
        let is_match = patterns.iter().any(|&pat| self.peek() == Some(pat));
        if is_match {
            self.advance();
        }
        is_match
    }

    // ** Recursive Descent **

    #[allow(clippy::similar_names)]
    fn recursive_descent_binary(
        &mut self,
        patterns: &[Token],
        descend_parse: impl Fn(&mut Self) -> Expr,
    ) -> Expr {
        let mut res = descend_parse(self);
        while self.advance_if_matches(patterns) {
            let lhs = Box::new(res);
            let op = self.previous().unwrap();
            let rhs = Box::new(descend_parse(self));
            res = Expr::Binary { lhs, op, rhs }
        }
        res
    }

    fn equality(&mut self) -> Expr {
        self.recursive_descent_binary(&[BangEqual, EqualEqual], Self::comparison)
    }

    fn comparison(&mut self) -> Expr {
        self.recursive_descent_binary(&[Greater, GreaterEqual, Less, LessEqual], Self::term)
    }

    fn term(&mut self) -> Expr {
        self.recursive_descent_binary(&[Plus, Minus], Self::factor)
    }

    fn factor(&mut self) -> Expr {
        self.recursive_descent_binary(&[Slash, Star], Self::unary)
    }

    fn unary(&mut self) -> Expr {
        if self.advance_if_matches(&[Plus, Minus]) {
            let op = self.previous().unwrap();
            let rhs = Box::new(self.unary());
            return Expr::Unary { op, rhs };
        }
        primary()
    }

    fn primary(&mut self) -> Expr {
        use Expr::*;

        match () {
            _ if self.advance_if_matches(&[False]) => Literal(Lit::Bool(false)),
            _ if self.advance_if_matches(&[True]) => Literal(Lit::Bool(true)),
            _ if self.advance_if_matches(&[Nil]) => Literal(Lit::Nil),
            _ if self.advance_if_matches(&[Number]) => {
                Literal(Lit::Number(self.peek().unwrap().span()))
            }
        }
    }
}

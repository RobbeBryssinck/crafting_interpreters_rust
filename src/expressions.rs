use crate::scanner::{Token, Literal};

#[derive(Clone)]
pub enum Expr {
	Assign {
		name: Token,
		value: Box<Expr>,
	},

	Binary {
		left: Box<Expr>,
		operator: Token,
		right: Box<Expr>,
	},

	Call {
		callee: Box<Expr>,
		paren: Token,
		arguments: Vec<Expr>,
	},

    Get {
        object: Box<Expr>,
        name: Token,
    },

    Grouping {
        expression: Box<Expr>,
    },

    Literal {
        value: Literal,
    },

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },

    Super {
        keyword: Token,
        method: Token,
    },

    This {
        keyword: Token,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable {
        name: Token,
    },
}
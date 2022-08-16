use crate::scanner::{Token, Literal};

#[derive(Debug, PartialEq)]
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

    /*
	Call {
		callee: Box<Expr>,
		paren: Token,
		arguments: Vec<Expr>,
	},

    Get {
        object: Box<Expr>,
        name: Token,
    },
     */

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

    /*
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
     */

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable {
        name: Token,
    },
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },

    /*
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>, // TODO: enforce Stmt::Function somehow
    },

    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
     */

    Expression {
        expression: Expr,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    Print {
        expression: Expr,
    },

    /*
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
     */

    Variable {
        name: Token,
        initializer: Option<Expr>,
    },

    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
use crate::scanner::{Token, Literal};

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

pub struct StmtFunction {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },

    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<StmtFunction>,
    },

    Expression {
        expression: Expr,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Stmt>,
    },

    Print {
        expression: Expr,
    },

    Return {
        keyword: Token,
        value: Option<Expr>,
    },

    Variable {
        name: Token,
        initializer: Option<Expr>,
    },

    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
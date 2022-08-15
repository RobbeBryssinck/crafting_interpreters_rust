use crate::scanner::{Token, TokenType, Literal};
use crate::syntax::{Expr, Stmt};

pub struct Parser {
    pub tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ()> {
        let mut is_error = false;
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    is_error = true;
                    println!("{}", e);
                    self.synchronize();
                }
            }
        }

        if is_error {
            Err(())
        } else {
            Ok(statements)
        }
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = match self.consume(TokenType::Identifier) {
            Some(token) => token,
            None => { return Err(self.generate_error("Expect variable name.")); }
        };

        let mut initializer: Option<Expr> = None;
        if self.match_tokens(&[TokenType::Equal]) {
            initializer = match self.expression() {
                Ok(expr) => Some(expr),
                Err(e) => { return Err(e); }
            };
        }

        match self.consume(TokenType::Semicolon) {
            Some(_token) => {},
            None => { return Err(self.generate_error("Expect ';' after variable decleration.")); }
        }

        Ok(Stmt::Variable { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_tokens(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_tokens(&[TokenType::LeftBrace]) {
            let statements = match self.block() {
                Ok(statements) => statements,
                Err(e) => { return Err(e); }
            };

            Ok(Stmt::Block { statements })
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = match self.expression() {
            Ok(expr) => expr,
            Err(e) => { return Err(e); }
        };

        match self.consume(TokenType::Semicolon) {
            Some(_token) => Ok(Stmt::Print { expression: value }),
            None => Err(self.generate_error("Expect ';' after value."))
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let declaration = match self.declaration() {
                Ok(declaration) => declaration,
                Err(e) => { return Err(e); }
            };

            statements.push(declaration);
        }

        match self.consume(TokenType::RightBrace) {
            Some(_token) => Ok(statements),
            None => Err(self.generate_error("Expect '}' after block."))
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let value = match self.expression() {
            Ok(expr) => expr,
            Err(e) => { return Err(e); }
        };

        match self.consume(TokenType::Semicolon, ) {
            Some(_token) => Ok(Stmt::Expression { expression: value }),
            None => Err(self.generate_error("Expect ';' after value."))
        }
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;

        if self.match_tokens(&[TokenType::Equal]) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign { name, value: Box::new(value) });
                },
                _ => { return Err(self.generate_error("Invalid assignment target.")); }
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_tokens(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary { 
                left: Box::from(expr), 
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary { 
                left: Box::from(expr), 
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary { 
                left: Box::from(expr), 
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary { 
                operator,
                right: Box::from(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::False]) {
            Ok(Expr::Literal { value: Literal::Bool(false) })
        } else if self.match_tokens(&[TokenType::True]) {
            Ok(Expr::Literal { value: Literal::Bool(true) })
        } else if self.match_tokens(&[TokenType::Nil]) {
            Ok(Expr::Literal { value: Literal::Nil })
        } else if self.match_tokens(&[TokenType::Number, TokenType::String]) {
            Ok(Expr::Literal { value: self.previous().clone().literal.unwrap() })
        } else if self.match_tokens(&[TokenType::Identifier]) {
            Ok(Expr::Variable { name: self.previous().clone() })
        } else if self.match_tokens(&[TokenType::LeftParen]) {
            let expr = self.expression()?;

            match self.consume(TokenType::RightParen) {
                Some(_token) => Ok(Expr::Grouping { expression: Box::new(expr) }),
                None => { return Err(self.generate_error("Expect ')' after expression.")); }
            }
        } else {
            Err(self.generate_error("Primary token not found."))
        }
    }

    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().token_type == token_type;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, token_type: TokenType) -> Option<Token> {
        if self.check(token_type) {
            Some(self.advance().clone())
        } else {
            None
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class | 
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current-1]
    }

    fn generate_error(&mut self, message: &str) -> String {
        let line = self.previous().line;
        format!("[line {line}] Error: {message}")
    }
}
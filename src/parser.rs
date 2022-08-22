use crate::scanner::{Token, TokenType, Literal};
use crate::syntax::{Expr, Stmt};

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Stmt>, ()> {
    let mut parser_runner = Parser::new(tokens);
    parser_runner.parse()
}

pub struct Parser {
    pub tokens: Vec<Token>,
    current: usize,
    loop_count: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_count: 0,
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
        } else if self.match_tokens(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_tokens(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_tokens(&[TokenType::Break]) {
            self.break_statement()
        } else if self.match_tokens(&[TokenType::If]) {
            self.if_statement()
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

    fn while_statement(&mut self) -> Result<Stmt, String> {
        match self.consume(TokenType::LeftParen) {
            Some(_) => {},
            None => { return Err(self.generate_error("Expect '(' after 'while'.")); }
        }

        let condition = self.expression()?;

        match self.consume(TokenType::RightParen) {
            Some(_) => {},
            None => { return Err(self.generate_error("Expect ')' after condition.")); }
        }

        self.loop_count += 1;
        let body = self.statement()?;
        self.loop_count -= 1;

        Ok(Stmt::While { condition, body: Box::new(body) })
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        match self.consume(TokenType::LeftParen) {
            Some(_) => {},
            None => { return Err(self.generate_error("Expect '(' after 'for'.")); }
        }

        let mut initializer: Option<Stmt> = None;
        if self.match_tokens(&[TokenType::Semicolon]) {
            // Do nothing, initializer is already None
        } else if self.match_tokens(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }

        match self.consume(TokenType::Semicolon) {
            Some(_) => {},
            None => { return Err(self.generate_error("Expect ';' after loop condition.")); }
        }

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }

        match self.consume(TokenType::RightParen) {
            Some(_) => {},
            None => { return Err(self.generate_error("Expect ')' after for clauses.")); }
        }

        self.loop_count += 1;
        let mut body = self.statement()?;
        self.loop_count -= 1;

        if increment.is_some() {
            body = Stmt::Block { statements: vec![body, Stmt::Expression { expression: increment.unwrap() }] }
        }

        if condition.is_none() {
            condition = Some(Expr::Literal { value: Literal::Bool(true) });
        }
        body = Stmt::While { condition: condition.unwrap(), body: Box::new(body) };

        if initializer.is_some() {
            body = Stmt::Block { statements: vec![initializer.unwrap(), body] };
        }

        Ok(body)
    }

    fn break_statement(&mut self) -> Result<Stmt, String> {
        if !self.is_in_loop() {
            return Err(self.generate_error("'break' statement must be in a loop block."));
        }

        match self.consume(TokenType::Semicolon) {
            Some(_token) => Ok(Stmt::Break {}),
            None => Err(self.generate_error("Expect ';' after 'break'."))
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        match self.consume(TokenType::LeftParen) {
            Some(_token) => {},
            None => {return Err(self.generate_error("Expect '(' after 'if'.")); }
        }

        let condition = self.expression()?;

        match self.consume(TokenType::RightParen) {
            Some(_token) => {},
            None => {return Err(self.generate_error("Expect ')' after if condition.")); }
        }

        let then_branch = self.statement()?;
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.match_tokens(&[TokenType::Else]) {
            else_branch = match self.statement() {
                Ok(statement) => Some(Box::new(statement)),
                Err(e) => { return Err(e); }
            }
        }

        Ok(Stmt::If { condition, then_branch: Box::new(then_branch), else_branch })
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
        let expr = self.or()?;

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

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_tokens(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical { left: Box::new(expr), operator, right: Box::new(right) }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_tokens(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical { left: Box::new(expr), operator, right: Box::new(right) }
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_tokens(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen) {
            if arguments.len() >= 255 {
                return Err("Can't have more than 255 arguments.".to_string());
            }

            loop {
                arguments.push(self.expression()?);
                if self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = match self.consume(TokenType::RightParen) {
            Some(token) => token,
            None => { return Err("Expect ')' after arguments.".to_string()); }
        };

        Ok(Expr::Call { callee: Box::from(callee), paren, arguments })
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
    
    fn is_in_loop(&self) -> bool {
        self.loop_count != 0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::Var,
                lexeme: String::from("var"),
                literal: None,
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: String::from("a"),
                literal: Some(Literal::Identifier("a".to_string())),
                line: 1,
            },
            Token {
                token_type: TokenType::Equal,
                lexeme: String::from("="),
                literal: None,
                line: 1,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("5"),
                literal: Some(Literal::Number(5.0)),
                line: 1,
            },
            Token {
                token_type: TokenType::Semicolon,
                lexeme: String::from(";"),
                literal: None,
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(";"),
                literal: None,
                line: 1,
            },
        ];

        let cmp_statements: Vec<Stmt> = vec![
            Stmt::Variable {
                name: tokens[1].clone(),
                initializer: Some(Expr::Literal { value: Literal::Number(5.0) }),
            },
        ];

        let statements = parse_tokens(tokens).unwrap();

        assert_eq!(statements.len(), 1);
        assert_eq!(statements, cmp_statements);
    }
}
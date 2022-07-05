use std::collections::HashMap;

use crate::error_reporter;

#[derive(Copy, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EOF
}

pub enum Literal {
    Identifier(String),
    Str(String),
    Number(f64),
}

pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: i32
}

struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
    keywords: HashMap<&'static str, TokenType>
}

impl Scanner {
    fn new(source: &str) -> Self {
        let mut s = Self {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
            keywords: HashMap::new()
        };

        s.fill_keywords();

        s
    }

    fn fill_keywords(&mut self) {
        self.keywords.insert("and", TokenType::And);
        self.keywords.insert("class", TokenType::Class);
        self.keywords.insert("else", TokenType::Else);
        self.keywords.insert("false", TokenType::False);
        self.keywords.insert("for", TokenType::For);
        self.keywords.insert("fun", TokenType::Fun);
        self.keywords.insert("if", TokenType::If);
        self.keywords.insert("nil", TokenType::Nil);
        self.keywords.insert("or", TokenType::Or);
        self.keywords.insert("print", TokenType::Print);
        self.keywords.insert("return", TokenType::Return);
        self.keywords.insert("super", TokenType::Super);
        self.keywords.insert("this", TokenType::This);
        self.keywords.insert("true", TokenType::True);
        self.keywords.insert("var", TokenType::Var);
        self.keywords.insert("while", TokenType::While);
    }

    fn get_keyword(&self, identifier: &str) -> Option<&TokenType> {
        self.keywords.get(identifier)
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.scan_token();
        }

        self.add_token(TokenType::EOF);
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let character = self.advance();

        match character {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.check_next('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            },
            '=' => {
                if self.check_next('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            },
            '<' => {
                if self.check_next('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            },
            '>' => {
                if self.check_next('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            },
            '/' => {
                if self.check_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            },
            ' ' | '\r' | '\t' => {},
            '\n' => self.line += 1,
            '"' => self.scan_string(),
            _ => {
                if character.is_digit(10) {
                    self.scan_number();
                } else if character.is_alphabetic() {
                    self.scan_identifier();
                } else {
                    error_reporter::error(self.line, "Unknown character");
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let next = self.source[self.current as usize];
        self.current += 1;
        next
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None)
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = String::from_iter(self.source[self.start..self.current].iter());

        self.tokens.push(Token{token_type, lexeme, literal, line: self.line})
    }

    fn check_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;

        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        } else {
            return self.source[self.current];
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        } else {
            return self.source[self.current+1];
        }
    }

    fn scan_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error_reporter::error(self.line, "Unterminated string.");
            return;
        }

        self.advance();

        let text = String::from_iter(self.source[self.start+1..self.current-1].iter());
        self.add_token_literal(TokenType::String, Some(Literal::Str(text)));
    }

    fn scan_number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the "."
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let num = String::from_iter(self.source[self.start..self.current].iter()).parse::<f64>().unwrap();
        self.add_token_literal(TokenType::Number, Some(Literal::Number(num)));
    }

    fn scan_identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let lexeme = String::from_iter(self.source[self.start..self.current].iter());
        
        match self.keywords.get(lexeme.as_str()) {
            Some(&token_type) => self.add_token(token_type),
            _ => self.add_token(TokenType::Identifier)
        }
    }
}

pub fn scan_tokens(source: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(source);

    scanner.scan_tokens();

    scanner.tokens
}
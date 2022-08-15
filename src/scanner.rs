use std::collections::HashMap;

pub fn scan_tokens(source: &str) -> Result<Vec<Token>, ()> {
    let mut scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(_) => Ok(scanner.tokens),
        Err(_) => Err(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Clone)]
pub enum Literal {
    Identifier(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Nil
}

#[derive(Clone)]
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
    is_error: bool,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
            is_error: false,
            keywords: HashMap::from([
                (String::from("and"), TokenType::And),
                (String::from("class"), TokenType::Class),
                (String::from("else"), TokenType::Else),
                (String::from("false"), TokenType::False),
                (String::from("fun"), TokenType::Fun),
                (String::from("for"), TokenType::For),
                (String::from("if"), TokenType::If),
                (String::from("nil"), TokenType::Nil),
                (String::from("or"), TokenType::Or),
                (String::from("print"), TokenType::Print),
                (String::from("return"), TokenType::Return),
                (String::from("super"), TokenType::Super),
                (String::from("this"), TokenType::This),
                (String::from("true"), TokenType::True),
                (String::from("var"), TokenType::Var),
                (String::from("while"), TokenType::While),
            ])
        }
    }

    fn scan_tokens(&mut self) -> Result<(), ()> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.add_token(TokenType::EOF);

        if !self.is_error {
            Ok(())
        } else {
            Err(())
        }
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
                } else if character.is_alphabetic() || character == '_' {
                    self.scan_identifier();
                } else {
                    self.report_error("unknown character.");
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
            self.report_error("unterminated string.");
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

        let literal = String::from_iter(self.source[self.start..self.current].iter()).parse::<f64>();

        self.add_token_literal(TokenType::Number, Some(Literal::Number(literal.unwrap())));
    }

    fn scan_identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = String::from_iter(self.source[self.start..self.current].iter());
        match self.keywords.get(&text) {
            Some(&token_type) => self.add_token(token_type),
            None => self.add_token_literal(TokenType::Identifier, Some(Literal::Identifier(text))),
        }
    }

    fn report_error(&mut self, message: &str) {
        self.is_error = true;
        let line = self.line;
        println!("[line {line}] Error: {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment() {
        let source = "var a = 5;";
        let tokens = scan_tokens(source).unwrap();

        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::Var);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn number_addition() {
        let source = "5 + 2;";
        let tokens = scan_tokens(source).unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[1].token_type, TokenType::Plus);
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[3].token_type, TokenType::Semicolon);
        assert_eq!(tokens[4].token_type, TokenType::EOF);
    }

    #[test]
    fn string_addition() {
        let source = "\"hello \" + \"world\";";
        let tokens = scan_tokens(source).unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[1].token_type, TokenType::Plus);
        assert_eq!(tokens[2].token_type, TokenType::String);
        assert_eq!(tokens[3].token_type, TokenType::Semicolon);
        assert_eq!(tokens[4].token_type, TokenType::EOF);
    }

    #[test]
    fn subtraction() {
        let source = "5 - 2;";
        let tokens = scan_tokens(source).unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[3].token_type, TokenType::Semicolon);
        assert_eq!(tokens[4].token_type, TokenType::EOF);
    }

    #[test]
    fn floating_points() {
        let source = "5.4 - 2.03;";
        let tokens = scan_tokens(source).unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Number);
        assert_eq!(tokens[3].token_type, TokenType::Semicolon);
        assert_eq!(tokens[4].token_type, TokenType::EOF);
    }
}
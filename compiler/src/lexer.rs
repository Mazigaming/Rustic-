use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Eof,
    Error,

    // Keywords
    Fn,
    Let,
    Mut,
    Const,
    If,
    Else,
    For,
    While,
    Return,
    Struct,
    Import,
    As,
    Type,
    Trait,
    Impl,
    Pub,
    Mod,
    Crate,
    Self_,
    Super,
    Use,
    In,
    Match,
    Branch,
    Default,

    // Literals
    Number(f64),
    StringLiteral(String),
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    EqEq,
    BangEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    And,
    Or,
    Xor,
    Not,
    BitAnd,
    BitOr,
    Arrow,
    DotDot,
    Dot,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Increment,
    Decrement,

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semi,
    Pound,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: (usize, usize),
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    pos: usize,
    input: &'a str,
}

pub fn tokenize(input: &str) -> Vec<SpannedToken> {
    let mut lexer = Lexer {
        chars: input.chars().peekable(),
        pos: 0,
        input,
    };
    let mut tokens = Vec::new();

    loop {
        let start = lexer.pos;
        let token = lexer.next_token();
        let end = lexer.pos;
        tokens.push(SpannedToken {
            token,
            span: (start, end),
        });
        if matches!(tokens.last().map(|t| &t.token), Some(Token::Eof)) {
            break;
        }
    }

    tokens
}

impl<'a> Lexer<'a> {
    fn next_token(&mut self) -> Token {
        loop {
            match self.chars.next() {
                None => return Token::Eof,
                Some(c) if c.is_whitespace() => {
                    self.pos += c.len_utf8();
                    continue;
                }
                Some('/') if self.peek() == Some('/') => {
                    self.chars.next();
                    self.pos += 2;
                    while let Some(c) = self.chars.next() {
                        self.pos += c.len_utf8();
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                Some(c) => return self.token_from_char(c),
            }
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn token_from_char(&mut self, c: char) -> Token {
        match c {
            '(' => {
                self.pos += 1;
                Token::LParen
            }
            ')' => {
                self.pos += 1;
                Token::RParen
            }
            '[' => {
                self.pos += 1;
                Token::LBracket
            }
            ']' => {
                self.pos += 1;
                Token::RBracket
            }
            '{' => {
                self.pos += 1;
                Token::LBrace
            }
            '}' => {
                self.pos += 1;
                Token::RBrace
            }
            ',' => {
                self.pos += 1;
                Token::Comma
            }
            ':' => {
                self.pos += 1;
                if self.peek() == Some('>') {
                    self.chars.next();
                    self.pos += 1;
                    Token::Arrow
                } else {
                    Token::Colon
                }
            }
            ';' => {
                self.pos += 1;
                Token::Semi
            }
            '#' => {
                self.pos += 1;
                Token::Pound
            }
            '+' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::PlusEq
                } else if self.peek() == Some('+') {
                    self.chars.next();
                    self.pos += 1;
                    Token::Increment
                } else {
                    Token::Plus
                }
            }
            '-' => {
                self.pos += 1;
                if self.peek() == Some('>') {
                    self.chars.next();
                    self.pos += 1;
                    Token::Arrow
                } else if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::MinusEq
                } else if self.peek() == Some('-') {
                    self.chars.next();
                    self.pos += 1;
                    Token::Decrement
                } else {
                    Token::Minus
                }
            }
            '*' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::StarEq
                } else {
                    Token::Star
                }
            }
            '/' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::SlashEq
                } else {
                    Token::Slash
                }
            }
            '%' => {
                self.pos += 1;
                Token::Percent
            }
            '=' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::EqEq
                } else {
                    Token::Assign
                }
            }
            '!' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::BangEq
                } else {
                    Token::Not
                }
            }
            '<' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::LessEq
                } else {
                    Token::Less
                }
            }
            '>' => {
                self.pos += 1;
                if self.peek() == Some('=') {
                    self.chars.next();
                    self.pos += 1;
                    Token::GreaterEq
                } else {
                    Token::Greater
                }
            }
            '&' => {
                self.pos += 1;
                if self.peek() == Some('&') {
                    self.chars.next();
                    self.pos += 1;
                    Token::And
                } else {
                    Token::BitAnd
                }
            }
            '|' => {
                self.pos += 1;
                if self.peek() == Some('|') {
                    self.chars.next();
                    self.pos += 1;
                    Token::Or
                } else {
                    Token::BitOr
                }
            }
            '^' => {
                self.pos += 1;
                Token::Xor
            }
            '.' => {
                self.pos += 1;
                if self.peek() == Some('.') {
                    self.chars.next();
                    self.pos += 1;
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            '0'..='9' => {
                let mut num_str = String::new();
                num_str.push(c);
                self.pos += 1;
                while let Some(&next) = self.chars.peek() {
                    if next.is_ascii_digit() {
                        let ch = self.chars.next().unwrap();
                        num_str.push(ch);
                        self.pos += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                if self.peek() == Some('.') && self.input.as_bytes().get(self.pos) != Some(&b'.') {
                    let ch = self.chars.next().unwrap();
                    num_str.push(ch);
                    self.pos += 1;
                    while let Some(&next) = self.chars.peek() {
                        if next.is_ascii_digit() {
                            let ch = self.chars.next().unwrap();
                            num_str.push(ch);
                            self.pos += ch.len_utf8();
                        } else {
                            break;
                        }
                    }
                }
                return match num_str.parse::<f64>() {
                    Ok(n) => Token::Number(n),
                    Err(_) => Token::Error,
                };
            }
            '"' => {
                self.pos += 1;
                let mut s = String::new();
                while let Some(ch) = self.chars.next() {
                    self.pos += ch.len_utf8();
                    if ch == '"' {
                        break;
                    }
                    if ch == '\\' {
                        if let Some(escaped) = self.chars.next() {
                            self.pos += escaped.len_utf8();
                            s.push(match escaped {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                '\\' => '\\',
                                '"' => '"',
                                _ => escaped,
                            });
                        }
                    } else {
                        s.push(ch);
                    }
                }
                return Token::StringLiteral(s);
            }
            _ if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = String::new();
                ident.push(c);
                self.pos += c.len_utf8();
                while let Some(&next) = self.chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        let ch = self.chars.next().unwrap();
                        ident.push(ch);
                        self.pos += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                return match ident.as_str() {
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "mut" => Token::Mut,
                    "const" => Token::Const,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "for" => Token::For,
                    "while" => Token::While,
                    "return" => Token::Return,
                    "struct" => Token::Struct,
                    "import" => Token::Import,
                    "as" => Token::As,
                    "type" => Token::Type,
                    "trait" => Token::Trait,
                    "impl" => Token::Impl,
                    "pub" => Token::Pub,
                    "mod" => Token::Mod,
                    "crate" => Token::Crate,
                    "self" => Token::Self_,
                    "super" => Token::Super,
                    "use" => Token::Use,
                    "in" => Token::In,
                    "match" => Token::Match,
                    "branch" => Token::Branch,
                    "default" => Token::Default,
                    _ => Token::Ident(ident),
                };
            }
            _ => {
                self.pos += 1;
                Token::Error
            }
        }
    }
}

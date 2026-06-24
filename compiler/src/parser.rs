use crate::lexer::{SpannedToken, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    Ident(String),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
    StructInit(String, Vec<(String, Expr)>),
    FieldAccess(Box<Expr>, String),
    Block(Vec<Stmt>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    Return(Option<Box<Expr>>),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
    BitAnd,
    BitOr,
    Assign,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
    Ref,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Option<Type>, Expr),
    Mut(String, Expr),
    Const(String, Option<Type>, Expr),
    Expr(Expr),
    Return(Option<Box<Expr>>),
    Item(Item),
    For(String, Box<Expr>, Box<Expr>, Expr),
    If(Box<Expr>, Expr, Option<Expr>),
    While(Box<Expr>, Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Prim(PrimType),
    Struct(String),
    Fn(Box<Type>, Vec<Type>),
    ForeignPtr(Box<Type>),
    Slice(Box<Type>),
    Ptr(Box<Type>),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimType {
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Float32,
    Float64,
    Bool,
    Char,
    String,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn {
        name: String,
        params: Vec<(String, Type)>,
        ret: Type,
        body: Expr,
    },
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    },
    Import {
        path: String,
        alias: Option<String>,
    },
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Item>, String> {
        let mut items = Vec::new();
        while !self.at_end() {
            let item = self.parse_item()?;
            items.push(item);
        }
        Ok(items)
    }

    fn at_end(&self) -> bool {
        self.peek().map_or(true, |t| matches!(t.token, Token::Eof))
    }

    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expect(&mut self, token: Token) -> Result<SpannedToken, String> {
        match self.advance() {
            Some(t) if t.token == token => Ok(t),
            Some(t) => Err(format!("Expected {:?}, got {:?}", token, t.token)),
            None => Err(format!("Expected {:?}, got EOF", token)),
        }
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        if self.peek_token(Token::Fn) {
            self.advance();
            self.parse_fn()
        } else if self.peek_token(Token::Struct) {
            self.advance();
            self.parse_struct()
        } else if self.peek_token(Token::Import) {
            self.advance();
            self.parse_import()
        } else {
            Err(format!(
                "Expected item (fn, struct, import), got {:?}",
                self.peek().map(|t| &t.token)
            ))
        }
    }

    fn peek_token(&self, token: Token) -> bool {
        match self.peek() {
            Some(t) => t.token == token,
            None => false,
        }
    }

    fn parse_fn(&mut self) -> Result<Item, String> {
        let name = self.parse_ident()?;
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        if !self.peek_token(Token::RParen) {
            loop {
                let pname = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ptype = self.parse_type()?;
                params.push((pname, ptype));
                if !self.peek_token(Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(Token::RParen)?;
        self.expect(Token::Arrow)?;
        let ret = self.parse_type()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        Ok(Item::Fn {
            name,
            params,
            ret,
            body,
        })
    }

    fn parse_struct(&mut self) -> Result<Item, String> {
        let name = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        loop {
            if self.peek_token(Token::RBrace) {
                break;
            }
            let fname = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let ftype = self.parse_type()?;
            fields.push((fname, ftype));
            if self.peek_token(Token::Comma) {
                self.advance();
            }
        }
        self.advance();
        Ok(Item::Struct { name, fields })
    }

    fn parse_import(&mut self) -> Result<Item, String> {
        let path = self.parse_string()?;
        let alias = if self.peek_token(Token::As) {
            self.advance();
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.expect(Token::Semi)?;
        Ok(Item::Import { path, alias })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if self.peek_token(Token::Type) {
            self.advance();
            return Ok(Type::Prim(PrimType::Int));
        }
        let ident = self.parse_ident()?;
        if self.peek_token(Token::LBracket) {
            self.advance();
            self.expect(Token::RBracket)?;
            return Ok(Type::Slice(Box::new(Type::Struct(ident))));
        }
        if self.peek_token(Token::Star) {
            self.advance();
            return Ok(Type::Ptr(Box::new(Type::Struct(ident))));
        }
        Ok(Type::Struct(ident))
    }

    fn parse_block(&mut self) -> Result<Expr, String> {
        let mut stmts = Vec::new();
        while !self.peek_token(Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.advance();
        Ok(Expr::Block(stmts))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.peek_token(Token::Let) {
            self.advance();
            let mut_ty = self.peek_token(Token::Mut);
            if mut_ty {
                self.advance();
            }
            let name = self.parse_ident()?;
            if self.peek_token(Token::Colon) {
                self.advance();
                let ty = self.parse_type()?;
                self.expect(Token::Assign)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semi)?;
                return Ok(Stmt::Let(name, Some(ty), expr));
            } else {
                self.expect(Token::Assign)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semi)?;
                return Ok(Stmt::Let(
                    name,
                    if mut_ty {
                        Some(Type::Prim(PrimType::Int))
                    } else {
                        None
                    },
                    expr,
                ));
            }
        }
        if self.peek_token(Token::Const) {
            self.advance();
            let name = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            self.expect(Token::Assign)?;
            let expr = self.parse_expr()?;
            self.expect(Token::Semi)?;
            return Ok(Stmt::Const(name, Some(ty), expr));
        }
        if self.peek_token(Token::Return) {
            self.advance();
            if self.peek_token(Token::RBrace) || self.at_end() {
                return Ok(Stmt::Return(None));
            }
            let expr = self.parse_expr()?;
            if self.peek_token(Token::Semi) {
                self.advance();
            }
            return Ok(Stmt::Return(Some(Box::new(expr))));
        }
        if self.peek_token(Token::For) {
            self.advance();
            let name = self.parse_ident()?;
            self.expect(Token::In)?;
            let start = self.parse_expr()?;
            self.expect(Token::DotDot)?;
            let end = self.parse_expr()?;
            self.expect(Token::LBrace)?;
            let body = self.parse_block()?;
            return Ok(Stmt::For(name, Box::new(start), Box::new(end), body));
        }
        if self.peek_token(Token::If) {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(Token::LBrace)?;
            let then_branch = self.parse_block()?;
            let else_branch = if self.peek_token(Token::Else) {
                self.advance();
                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                Some(body)
            } else {
                None
            };
            return Ok(Stmt::If(Box::new(cond), then_branch, else_branch));
        }
        if self.peek_token(Token::While) {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(Token::LBrace)?;
            let body = self.parse_block()?;
            return Ok(Stmt::While(Box::new(cond), body));
        }
        let expr = self.parse_assign()?;
        if self.peek_token(Token::Semi) {
            self.advance();
        }
        Ok(Stmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_or_expr()?;
        if self.peek_token(Token::Assign) {
            self.advance();
            let right = self.parse_assign()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::Assign, Box::new(right));
        }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and_expr()?;
        while self.peek_token(Token::Or) {
            self.advance();
            let op = BinOp::Or;
            let right = self.parse_and_expr()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_xor_expr()?;
        while self.peek_token(Token::And) {
            self.advance();
            let op = BinOp::And;
            let right = self.parse_xor_expr()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_xor_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bit_or_expr()?;
        while self.peek_token(Token::Xor) {
            self.advance();
            let op = BinOp::Xor;
            let right = self.parse_bit_or_expr()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_bit_or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bit_and_expr()?;
        while self.peek_token(Token::BitOr) {
            self.advance();
            let op = BinOp::BitOr;
            let right = self.parse_bit_and_expr()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_bit_and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.peek_token(Token::BitAnd) {
            self.advance();
            let op = BinOp::BitAnd;
            let right = self.parse_equality()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = if self.peek_token(Token::EqEq) {
                self.advance();
                BinOp::Eq
            } else if self.peek_token(Token::BangEq) {
                self.advance();
                BinOp::Ne
            } else {
                return Ok(left);
            };
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            let op = if self.peek_token(Token::Less) {
                self.advance();
                BinOp::Lt
            } else if self.peek_token(Token::LessEq) {
                self.advance();
                BinOp::Le
            } else if self.peek_token(Token::Greater) {
                self.advance();
                BinOp::Gt
            } else if self.peek_token(Token::GreaterEq) {
                self.advance();
                BinOp::Ge
            } else {
                return Ok(left);
            };
            let right = self.parse_term()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;
        loop {
            let op = if self.peek_token(Token::Plus) {
                self.advance();
                BinOp::Add
            } else if self.peek_token(Token::Minus) {
                self.advance();
                BinOp::Sub
            } else {
                return Ok(left);
            };
            let right = self.parse_factor()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.peek_token(Token::Star) {
                self.advance();
                BinOp::Mul
            } else if self.peek_token(Token::Slash) {
                self.advance();
                BinOp::Div
            } else if self.peek_token(Token::Percent) {
                self.advance();
                BinOp::Mod
            } else {
                return Ok(left);
            };
            let right = self.parse_unary()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.peek_token(Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)));
        }
        if self.peek_token(Token::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        if let Some(t) = self.peek().cloned() {
            if matches!(t.token, Token::Number(_)) {
                self.advance();
                if let Token::Number(n) = t.token {
                    if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                        return Ok(Expr::IntLiteral(n as i64));
                    }
                    return Ok(Expr::FloatLiteral(n));
                }
            }
            if matches!(t.token, Token::StringLiteral(_)) {
                self.advance();
                if let Token::StringLiteral(s) = t.token {
                    return Ok(Expr::StringLiteral(s));
                }
            }
        }
        let ident = self.parse_ident()?;
        let mut expr = Expr::Ident(ident);
        while self.peek_token(Token::Dot) {
            self.advance();
            let field = self.parse_ident()?;
            expr = Expr::FieldAccess(Box::new(expr), field);
        }
        if self.peek_token(Token::LParen) {
            self.advance();
            let mut args = Vec::new();
            if !self.peek_token(Token::RParen) {
                args.push(self.parse_expr()?);
                while self.peek_token(Token::Comma) {
                    self.advance();
                    args.push(self.parse_expr()?);
                }
                self.expect(Token::RParen)?;
            } else {
                self.advance();
            }
            return Ok(Expr::Call(
                if let Expr::Ident(name) = expr {
                    name
                } else {
                    return Err("Expected identifier for function call".to_string());
                },
                args,
            ));
        }
        Ok(expr)
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(SpannedToken {
                token: Token::Ident(s),
                ..
            }) => Ok(s),
            _ => Err("Expected identifier".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(SpannedToken {
                token: Token::StringLiteral(s),
                ..
            }) => Ok(s),
            _ => Err("Expected string".to_string()),
        }
    }
}

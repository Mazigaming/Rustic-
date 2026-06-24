pub mod lexer;
pub mod parser;
pub mod type_check;

pub use lexer::{tokenize, SpannedToken, Token};
pub use parser::{BinOp, Expr, Item, Parser, PrimType, Stmt, Type, UnaryOp};
pub use type_check::{TypeChecker, TypeError};

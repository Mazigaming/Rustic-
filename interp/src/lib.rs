pub use rustic_compiler::{Expr, Item, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(String, Vec<(String, Value)>),
    Func(Vec<String>, Box<Expr>, Vec<String>),
    ExternalPtr(*const u8),
    Undefined,
    Break,
    Continue,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Struct(a, f1), Value::Struct(b, f2)) => a == b && f1 == f2,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Break, Value::Break) => true,
            (Value::Continue, Value::Continue) => true,
            _ => false,
        }
    }
}

pub type Env = Vec<HashMap<String, Value>>;

const MAX_RECURSION_DEPTH: usize = 50;

pub fn eval_item(item: &Item, env: &mut Env) -> Result<(), String> {
    match item {
        Item::Fn {
            name,
            params,
            ret: _,
            body,
        } => {
            env.last_mut().unwrap().insert(
                name.clone(),
                Value::Func(
                    params.iter().map(|(n, _)| n.clone()).collect(),
                    Box::new(body.clone()),
                    Vec::new(),
                ),
            );
        }
        Item::Struct { name: _, fields: _ } => {}
        Item::Import { path: _, alias: _ } => {}
    }
    Ok(())
}

pub fn eval_expr_with_depth(expr: &Expr, env: &mut Env, depth: usize) -> Result<Value, String> {
    if depth > MAX_RECURSION_DEPTH {
        return Err("Maximum recursion depth exceeded".to_string());
    }
    eval_expr_impl(expr, env, depth)
}

fn eval_expr_impl(expr: &Expr, env: &mut Env, depth: usize) -> Result<Value, String> {
    match expr {
        Expr::IntLiteral(n) => Ok(Value::Int(*n)),
        Expr::FloatLiteral(n) => Ok(Value::Float(*n)),
        Expr::StringLiteral(s) => Ok(Value::String(s.clone())),
        Expr::Ident(name) => {
            for scope in env.iter().rev() {
                if let Some(v) = scope.get(name) {
                    return Ok(v.clone());
                }
            }
            Err(format!("Undefined variable: {}", name))
        }
        Expr::BinaryOp(left, op, right) => {
            let l = eval_expr_impl(left, env, depth)?;
            let r = eval_expr_impl(right, env, depth)?;
            match op {
                rustic_compiler::BinOp::Add => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                    _ => Err("Type error in +".to_string()),
                },
                rustic_compiler::BinOp::Sub => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                    _ => Err("Type error in -".to_string()),
                },
                rustic_compiler::BinOp::Mul => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                    _ => Err("Type error in *".to_string()),
                },
                rustic_compiler::BinOp::Div => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                    _ => Err("Type error in /".to_string()),
                },
                rustic_compiler::BinOp::Mod => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                    _ => Err("Type error in %".to_string()),
                },
                rustic_compiler::BinOp::Lt => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                    _ => Err("Type error in <".to_string()),
                },
                rustic_compiler::BinOp::Le => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                    _ => Err("Type error in <=".to_string()),
                },
                rustic_compiler::BinOp::Gt => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                    _ => Err("Type error in >".to_string()),
                },
                rustic_compiler::BinOp::Ge => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                    _ => Err("Type error in >=".to_string()),
                },
                rustic_compiler::BinOp::Eq => Ok(Value::Bool(l == r)),
                rustic_compiler::BinOp::Ne => Ok(Value::Bool(l != r)),
                rustic_compiler::BinOp::And => match (l, r) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    _ => Err("Type error in &&".to_string()),
                },
                rustic_compiler::BinOp::Or => match (l, r) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => Err("Type error in ||".to_string()),
                },
                rustic_compiler::BinOp::Xor => match (l, r) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a ^ b)),
                    _ => Err("Type error in ^".to_string()),
                },
                rustic_compiler::BinOp::BitAnd => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                    _ => Err("Type error in &".to_string()),
                },
                rustic_compiler::BinOp::BitOr => match (l, r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                    _ => Err("Type error in |".to_string()),
                },
                rustic_compiler::BinOp::Assign => {
                    if let Expr::Ident(name) = left.as_ref() {
                        for scope in env.iter_mut().rev() {
                            if scope.contains_key(name) {
                                scope.insert(name.clone(), r.clone());
                                return Ok(r);
                            }
                        }
                        return Err(format!("Undefined variable: {}", name));
                    }
                    Err("Invalid assignment target".to_string())
                }
            }
        }
        Expr::UnaryOp(op, expr) => {
            let v = eval_expr_impl(expr, env, depth)?;
            match op {
                rustic_compiler::UnaryOp::Neg => match v {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(n) => Ok(Value::Float(-n)),
                    _ => Err("Type error in -".to_string()),
                },
                rustic_compiler::UnaryOp::Not => match v {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err("Type error in !".to_string()),
                },
                _ => Err(format!("Unknown unary op: {:?}", op)),
            }
        }
        Expr::Call(name, args) => {
            let arg_vals: Result<Vec<Value>, _> =
                args.iter().map(|a| eval_expr_impl(a, env, depth)).collect();
            let arg_vals = arg_vals?;

            for scope in env.iter().rev() {
                if let Some(Value::Func(params, body, _)) = scope.get(name) {
                    let mut call_env = env.clone();
                    call_env.push(HashMap::new());
                    for (i, param) in params.iter().enumerate() {
                        call_env.last_mut().unwrap().insert(
                            param.clone(),
                            arg_vals.get(i).cloned().unwrap_or(Value::Undefined),
                        );
                    }
                    return eval_expr_impl(body, &mut call_env, depth + 1);
                }
            }
            Err(format!("Undefined function: {}", name))
        }
        Expr::StructInit(name, fields) => {
            let field_vals: Result<Vec<(String, Value)>, String> = fields
                .iter()
                .map(|(f, v)| Ok((f.clone(), eval_expr_impl(v, env, depth)?)))
                .collect();
            Ok(Value::Struct(name.clone(), field_vals?))
        }
        Expr::FieldAccess(obj, field) => {
            let o = eval_expr_impl(obj, env, depth)?;
            match o {
                Value::Struct(_, fields) => {
                    for (f, v) in fields {
                        if f == *field {
                            return Ok(v);
                        }
                    }
                    Err(format!("Field {} not found", field))
                }
                _ => Err("Cannot access field on non-struct".to_string()),
            }
        }
        Expr::Block(stmts) => {
            env.push(HashMap::new());
            let mut result = Value::Undefined;
            for stmt in stmts {
                match eval_stmt_impl(stmt, env, depth)? {
                    EvalResult::Return(v) => {
                        env.pop();
                        return Ok(v);
                    }
                    EvalResult::Value(v) => {
                        result = v;
                    }
                    EvalResult::Break | EvalResult::Continue => {
                        env.pop();
                        return Ok(result);
                    }
                }
            }
            env.pop();
            Ok(result)
        }
        Expr::If(cond, then_branch, else_branch) => {
            let c = eval_expr_impl(cond, env, depth)?;
            let should_run = match c {
                Value::Bool(b) => b,
                _ => return Err("Condition must be bool".to_string()),
            };
            if should_run {
                match eval_stmt_impl(then_branch, env, depth)? {
                    EvalResult::Return(v) => return Ok(v),
                    EvalResult::Value(v) => return Ok(v),
                    EvalResult::Break => return Ok(Value::Undefined),
                    EvalResult::Continue => return Ok(Value::Undefined),
                }
            } else if let Some(else_b) = else_branch {
                match eval_stmt_impl(else_b, env, depth)? {
                    EvalResult::Return(v) => return Ok(v),
                    EvalResult::Value(v) => return Ok(v),
                    EvalResult::Break => return Ok(Value::Undefined),
                    EvalResult::Continue => return Ok(Value::Undefined),
                }
            } else {
                Ok(Value::Undefined)
            }
        }
        Expr::While(cond, body) => {
            loop {
                let c = eval_expr_impl(cond, env, depth)?;
                let should_run = match c {
                    Value::Bool(b) => b,
                    _ => return Err("Condition must be bool".to_string()),
                };
                if !should_run {
                    break;
                }
                match eval_stmt_impl(body, env, depth)? {
                    EvalResult::Return(v) => return Ok(v),
                    EvalResult::Break => break,
                    _ => {}
                }
            }
            Ok(Value::Undefined)
        }
        Expr::Return(val) => {
            if let Some(v) = val {
                Ok(eval_expr_impl(v, env, depth)?)
            } else {
                Ok(Value::Undefined)
            }
        }
        Expr::Break => Err("Break outside loop".to_string()),
        Expr::Continue => Err("Continue outside loop".to_string()),
    }
}

pub fn eval_expr(expr: &Expr, env: &mut Env) -> Result<Value, String> {
    eval_expr_with_depth(expr, env, 0)
}

pub enum EvalResult {
    Value(Value),
    Return(Value),
    Break,
    Continue,
}

pub fn eval_stmt(stmt: &Stmt, env: &mut Env) -> Result<EvalResult, String> {
    eval_stmt_impl(stmt, env, 0)
}

fn eval_stmt_impl(stmt: &Stmt, env: &mut Env, depth: usize) -> Result<EvalResult, String> {
    match stmt {
        Stmt::Let(name, _ty, expr) => {
            let v = eval_expr_impl(expr, env, depth)?;
            env.last_mut().unwrap().insert(name.clone(), v);
            Ok(EvalResult::Value(Value::Undefined))
        }
        Stmt::Mut(name, expr) => {
            let v = eval_expr_impl(expr, env, depth)?;
            if let Some(scope) = env.last_mut() {
                scope.insert(name.clone(), v);
            }
            Ok(EvalResult::Value(Value::Undefined))
        }
        Stmt::Const(name, _ty, expr) => {
            let v = eval_expr_impl(expr, env, depth)?;
            env.last_mut().unwrap().insert(name.clone(), v);
            Ok(EvalResult::Value(Value::Undefined))
        }
        Stmt::Expr(expr) => eval_expr_impl(expr, env, depth).map(|v| EvalResult::Value(v)),
        Stmt::Return(val) => {
            if let Some(v) = val {
                Ok(EvalResult::Return(eval_expr_impl(v, env, depth)?))
            } else {
                Ok(EvalResult::Return(Value::Undefined))
            }
        }
        Stmt::Item(item) => {
            eval_item(item, env)?;
            Ok(EvalResult::Value(Value::Undefined))
        }
        Stmt::For(var, start, end, body) => {
            let s = eval_expr_impl(start, env, depth)?;
            let e = eval_expr_impl(end, env, depth)?;
            let start_val = match s {
                Value::Int(n) => n,
                _ => return Err("For loop start must be int".to_string()),
            };
            let end_val = match e {
                Value::Int(n) => n,
                _ => return Err("For loop end must be int".to_string()),
            };
            for i in start_val..end_val {
                env.last_mut().unwrap().insert(var.clone(), Value::Int(i));
                let _ = eval_expr_impl(body, env, depth)?;
            }
            Ok(EvalResult::Value(Value::Undefined))
        }
        Stmt::If(cond, then_branch, else_branch) => {
            let c = eval_expr_impl(cond, env, depth)?;
            let should_run = match c {
                Value::Bool(b) => b,
                _ => return Err("Condition must be bool".to_string()),
            };
            if should_run {
                match eval_expr_impl(then_branch, env, depth)? {
                    v => Ok(EvalResult::Value(v)),
                }
            } else if let Some(else_b) = else_branch {
                match eval_expr_impl(else_b, env, depth)? {
                    v => Ok(EvalResult::Value(v)),
                }
            } else {
                Ok(EvalResult::Value(Value::Undefined))
            }
        }
        Stmt::While(cond, body) => {
            loop {
                let c = eval_expr_impl(cond, env, depth)?;
                let should_run = match c {
                    Value::Bool(b) => b,
                    _ => return Err("Condition must be bool".to_string()),
                };
                if !should_run {
                    break;
                }
                match eval_expr_impl(body, env, depth)? {
                    Value::Undefined => {}
                    v => return Ok(EvalResult::Value(v)),
                }
            }
            Ok(EvalResult::Value(Value::Undefined))
        }
    }
}

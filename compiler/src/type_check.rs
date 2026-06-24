use crate::{BinOp, Expr, Item, PrimType, Stmt, Type, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    TypeMismatch { expected: String, got: String },
    InvalidOperand { op: String, type_name: String },
    ReturnTypeMismatch { expected: String, got: String },
    FieldNotFound { struct_name: String, field: String },
    NotAType(String),
}

pub type TypeResult<T> = Result<T, TypeError>;

pub struct TypeChecker {
    pub types: Vec<std::collections::HashMap<String, Type>>,
    pub functions: std::collections::HashMap<String, (Type, Vec<Type>)>,
    pub structs: std::collections::HashMap<String, Vec<(String, Type)>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            types: vec![std::collections::HashMap::new()],
            functions: std::collections::HashMap::new(),
            structs: std::collections::HashMap::new(),
        }
    }

    pub fn check_items(&mut self, items: &[Item]) -> Result<(), TypeError> {
        for item in items {
            self.check_item(item)?;
        }
        Ok(())
    }

    fn check_item(&mut self, item: &Item) -> TypeResult<()> {
        match item {
            Item::Fn {
                name,
                params,
                ret,
                body,
            } => {
                let param_types: Vec<Type> = params.iter().map(|(_, t)| t.clone()).collect();
                self.functions
                    .insert(name.clone(), (ret.clone(), param_types));
                self.types.push(std::collections::HashMap::new());
                for (param_name, param_type) in params {
                    self.types
                        .last_mut()
                        .unwrap()
                        .insert(param_name.clone(), param_type.clone());
                }
                let body_type = self.check_expr(body)?;
                self.types.pop();
                if body_type != *ret && *ret != Type::Prim(PrimType::Int) {
                    return Err(TypeError::ReturnTypeMismatch {
                        expected: format!("{}", ret),
                        got: format!("{}", body_type),
                    });
                }
            }
            Item::Struct { name, fields } => {
                self.structs.insert(name.clone(), fields.clone());
            }
            Item::Import { path: _, alias: _ } => {}
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr) -> TypeResult<Type> {
        match expr {
            Expr::IntLiteral(_) => Ok(Type::Prim(PrimType::Int)),
            Expr::FloatLiteral(_) => Ok(Type::Prim(PrimType::Float)),
            Expr::StringLiteral(_) => Ok(Type::Struct("String".to_string())),
            Expr::Ident(name) => {
                for scope in self.types.iter().rev() {
                    if let Some(t) = scope.get(name) {
                        return Ok(t.clone());
                    }
                }
                Err(TypeError::UndefinedVariable(name.clone()))
            }
            Expr::BinaryOp(left, op, right) => {
                let l = self.check_expr(left)?;
                let r = self.check_expr(right)?;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if l == Type::Prim(PrimType::Int) && r == Type::Prim(PrimType::Int) {
                            Ok(Type::Prim(PrimType::Int))
                        } else if l == Type::Prim(PrimType::Float)
                            && r == Type::Prim(PrimType::Float)
                        {
                            Ok(Type::Prim(PrimType::Float))
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: "Int or Float".to_string(),
                                got: format!("{} and {}", l, r),
                            })
                        }
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        if l == r {
                            Ok(Type::Prim(PrimType::Bool))
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: l.to_string(),
                                got: r.to_string(),
                            })
                        }
                    }
                    BinOp::Eq | BinOp::Ne => Ok(Type::Prim(PrimType::Bool)),
                    BinOp::And | BinOp::Or | BinOp::Xor => {
                        if l == Type::Prim(PrimType::Bool) && r == Type::Prim(PrimType::Bool) {
                            Ok(Type::Prim(PrimType::Bool))
                        } else {
                            Err(TypeError::TypeMismatch {
                                expected: "Bool".to_string(),
                                got: format!("{} and {}", l, r),
                            })
                        }
                    }
                    BinOp::BitAnd | BinOp::BitOr => {
                        if l == Type::Prim(PrimType::Int) && r == Type::Prim(PrimType::Int) {
                            Ok(Type::Prim(PrimType::Int))
                        } else {
                            Err(TypeError::InvalidOperand {
                                op: format!("{:?}", op),
                                type_name: format!("{} and {}", l, r),
                            })
                        }
                    }
                    BinOp::Assign => {
                        if let Expr::Ident(name) = left.as_ref() {
                            for scope in self.types.iter_mut().rev() {
                                if scope.contains_key(name) {
                                    return Ok(r.clone());
                                }
                            }
                            return Err(TypeError::UndefinedVariable(name.clone()));
                        }
                        Err(TypeError::NotAType("Invalid assignment target".to_string()))
                    }
                }
            }
            Expr::UnaryOp(op, expr) => {
                let t = self.check_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        if t == Type::Prim(PrimType::Int) || t == Type::Prim(PrimType::Float) {
                            Ok(t)
                        } else {
                            Err(TypeError::InvalidOperand {
                                op: "-".to_string(),
                                type_name: t.to_string(),
                            })
                        }
                    }
                    UnaryOp::Not => {
                        if t == Type::Prim(PrimType::Bool) {
                            Ok(Type::Prim(PrimType::Bool))
                        } else {
                            Err(TypeError::InvalidOperand {
                                op: "!".to_string(),
                                type_name: t.to_string(),
                            })
                        }
                    }
                    _ => Err(TypeError::NotAType(format!("Unknown unary op: {:?}", op))),
                }
            }
            Expr::Call(name, args) => {
                let (ret_type, param_types) = self
                    .functions
                    .get(name)
                    .cloned()
                    .ok_or_else(|| TypeError::UndefinedFunction(name.clone()))?;
                if args.len() != param_types.len() {
                    return Err(TypeError::TypeMismatch {
                        expected: format!("{} args", param_types.len()),
                        got: format!("{} args", args.len()),
                    });
                }
                for (arg, expected) in args.iter().zip(param_types.iter()) {
                    let arg_type = self.check_expr(arg)?;
                    if arg_type != *expected {
                        return Err(TypeError::TypeMismatch {
                            expected: expected.to_string(),
                            got: arg_type.to_string(),
                        });
                    }
                }
                Ok(ret_type)
            }
            Expr::StructInit(name, fields) => {
                let field_types = self.structs.get(name).cloned().ok_or_else(|| {
                    TypeError::UndefinedVariable(format!("Unknown struct: {}", name))
                })?;
                for (field_name, field_expr) in fields {
                    let field_type = self.check_expr(field_expr)?;
                    if let Some(expected) = field_types
                        .iter()
                        .find(|(n, _)| n == field_name)
                        .map(|(_, t)| t)
                    {
                        if field_type != *expected {
                            return Err(TypeError::TypeMismatch {
                                expected: expected.to_string(),
                                got: field_type.to_string(),
                            });
                        }
                    } else {
                        return Err(TypeError::FieldNotFound {
                            struct_name: name.clone(),
                            field: field_name.clone(),
                        });
                    }
                }
                Ok(Type::Struct(name.clone()))
            }
            Expr::FieldAccess(obj, field) => {
                let obj_type = self.check_expr(obj)?;
                if let Type::Struct(struct_name) = obj_type {
                    if let Some(field_types) = self.structs.get(&struct_name) {
                        for (f, t) in field_types {
                            if f == field {
                                return Ok(t.clone());
                            }
                        }
                        Err(TypeError::FieldNotFound {
                            struct_name,
                            field: field.clone(),
                        })
                    } else {
                        Err(TypeError::UndefinedVariable(struct_name))
                    }
                } else {
                    Err(TypeError::NotAType(format!(
                        "Cannot access field on {}",
                        obj_type
                    )))
                }
            }
            Expr::Block(stmts) => {
                self.types.push(std::collections::HashMap::new());
                let mut result = Type::Prim(PrimType::Int);
                for stmt in stmts {
                    result = self.check_stmt(stmt)?;
                }
                self.types.pop();
                Ok(result)
            }
            Expr::If(cond, then_branch, else_branch) => {
                let cond_type = self.check_expr(cond)?;
                if cond_type != Type::Prim(PrimType::Bool) {
                    return Err(TypeError::TypeMismatch {
                        expected: "Bool".to_string(),
                        got: cond_type.to_string(),
                    });
                }
                let then_type = self.check_stmt(&**then_branch)?;
                if let Some(else_b) = else_branch {
                    let else_type = self.check_stmt(&**else_b)?;
                    if then_type != else_type {
                        return Err(TypeError::TypeMismatch {
                            expected: then_type.to_string(),
                            got: else_type.to_string(),
                        });
                    }
                }
                Ok(then_type)
            }
            Expr::While(cond, body) => {
                let cond_type = self.check_expr(cond)?;
                if cond_type != Type::Prim(PrimType::Bool) {
                    return Err(TypeError::TypeMismatch {
                        expected: "Bool".to_string(),
                        got: cond_type.to_string(),
                    });
                }
                self.check_stmt(&**body)?;
                Ok(Type::Prim(PrimType::Int))
            }
            Expr::Return(val) => {
                if let Some(v) = val {
                    self.check_expr(v)
                } else {
                    Ok(Type::Prim(PrimType::Int))
                }
            }
            Expr::Break | Expr::Continue => Ok(Type::Prim(PrimType::Int)),
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> TypeResult<Type> {
        match stmt {
            Stmt::Let(name, ty, expr) => {
                let expr_type = self.check_expr(expr)?;
                if let Some(expected) = ty {
                    if expr_type != *expected {
                        return Err(TypeError::TypeMismatch {
                            expected: expected.to_string(),
                            got: expr_type.to_string(),
                        });
                    }
                }
                self.types
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), expr_type);
                Ok(Type::Prim(PrimType::Int))
            }
            Stmt::Mut(name, expr) => {
                let expr_type = self.check_expr(expr)?;
                for scope in self.types.iter_mut().rev() {
                    if scope.contains_key(name) {
                        let val = expr_type.clone();
                        scope.insert(name.clone(), val);
                        return Ok(expr_type);
                    }
                }
                Err(TypeError::UndefinedVariable(name.clone()))
            }
            Stmt::Const(name, ty, expr) => {
                let expr_type = self.check_expr(expr)?;
                if let Some(expected) = ty {
                    if expr_type != *expected {
                        return Err(TypeError::TypeMismatch {
                            expected: expected.to_string(),
                            got: expr_type.to_string(),
                        });
                    }
                }
                self.types
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), expr_type);
                Ok(Type::Prim(PrimType::Int))
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Return(val) => {
                if let Some(v) = val {
                    self.check_expr(v)
                } else {
                    Ok(Type::Prim(PrimType::Int))
                }
            }
            Stmt::Item(item) => {
                self.check_item(item)?;
                Ok(Type::Prim(PrimType::Int))
            }
            Stmt::For(var, start, end, body) => {
                let start_type = self.check_expr(start)?;
                let end_type = self.check_expr(end)?;
                if start_type != Type::Prim(PrimType::Int) || end_type != Type::Prim(PrimType::Int)
                {
                    return Err(TypeError::TypeMismatch {
                        expected: "Int".to_string(),
                        got: format!("{} and {}", start_type, end_type),
                    });
                }
                self.types
                    .last_mut()
                    .unwrap()
                    .insert(var.clone(), Type::Prim(PrimType::Int));
                self.check_expr(body)?;
                Ok(Type::Prim(PrimType::Int))
            }
            Stmt::If(cond, then_branch, else_branch) => {
                let cond_type = self.check_expr(cond)?;
                if cond_type != Type::Prim(PrimType::Bool) {
                    return Err(TypeError::TypeMismatch {
                        expected: "Bool".to_string(),
                        got: cond_type.to_string(),
                    });
                }
                let then_type = self.check_expr(then_branch)?;
                if let Some(else_b) = else_branch {
                    let else_type = self.check_expr(else_b)?;
                    if then_type != else_type {
                        return Err(TypeError::TypeMismatch {
                            expected: then_type.to_string(),
                            got: else_type.to_string(),
                        });
                    }
                }
                Ok(then_type)
            }
            Stmt::While(cond, body) => {
                let cond_type = self.check_expr(cond)?;
                if cond_type != Type::Prim(PrimType::Bool) {
                    return Err(TypeError::TypeMismatch {
                        expected: "Bool".to_string(),
                        got: cond_type.to_string(),
                    });
                }
                self.check_expr(body)?;
                Ok(Type::Prim(PrimType::Int))
            }
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Prim(p) => write!(f, "{:?}", p),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Fn(args, ret) => write!(f, "fn({:?}) -> {:?}", args, ret),
            Type::ForeignPtr(t) => write!(f, "*{:?}", t),
            Type::Slice(t) => write!(f, "[{:?}]", t),
            Type::Ptr(t) => write!(f, "*{:?}", t),
            Type::Unit => write!(f, "()"),
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            TypeError::UndefinedFunction(name) => write!(f, "Undefined function: {}", name),
            TypeError::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            }
            TypeError::InvalidOperand { op, type_name } => {
                write!(f, "Invalid operand {} for type {}", op, type_name)
            }
            TypeError::ReturnTypeMismatch { expected, got } => write!(
                f,
                "Return type mismatch: expected {}, got {}",
                expected, got
            ),
            TypeError::FieldNotFound { struct_name, field } => {
                write!(f, "Field {} not found in struct {}", field, struct_name)
            }
            TypeError::NotAType(s) => write!(f, "{}", s),
        }
    }
}

use std::ops::Range;
use parser::Expr;

#[derive(Debug)]
pub enum Value { Int(i64), Boolean(bool), String(String), Ident(String) }

#[derive(Debug)]
pub enum IRKind {
    Define { name: String, type_hint: String, value: Box<Self> },
    Fun { name: String, return_type_hint: String, args: Vec<(String, String)>, body: Box<Self> },
    Call { name: String, args: Vec<Self> },
    Do { body: Vec<Self> },
    If { cond: Box<Self>, body: Box<Self>, else_body: Box<Self> },
    Value { value: Value },
    Binary { op: String, left: Box<Self>, right: Box<Self> },
    Return { value: Box<Self> },
}

#[derive(Debug)]
pub struct IR {
    pub kind: IRKind,
    pub span: Range<usize>
}

#[derive(Debug)]
pub struct LoweringError {
    pub span: Range<usize>,
    pub message: String
}

impl IR {
    pub fn new(kind: IRKind, span: Range<usize>) -> Self {
        Self { kind, span }
    }
}

pub fn ast_to_ir(ast: Vec<(Expr, Range<usize>)>) -> (Vec<IR>, Vec<LoweringError>) {
    let mut irs = Vec::new();
    let mut errors = Vec::new();
    for expr in ast {
        let ir_kind = expr_to_ir(&expr.0);
        if let Some(err) = ir_kind.1 {
            errors.push(err);
        } else {
            irs.push(IR::new(ir_kind.0.unwrap(), expr.1));
        }
    }
    (irs, errors)
}

pub fn expr_to_ir(expr: &Expr) -> (Option<IRKind>, Option<LoweringError>) {
    match expr {
        Expr::Let { name, type_hint, value } => {
            let value = expr_to_ir(&value.0);
            if let Some(err) = value.1 {
                // Return error
                return (None, Some(err));
            } else {
                let value = value.0.unwrap(); // Unwrapping because it should always be Some
                let ir_kind = IRKind::Define { name: name.clone(), type_hint: type_hint.clone(), value: Box::new(value) };
                return (Some(ir_kind), None);
            }
        },
        Expr::Call { name, args } => {
            let name = match &name.0 {
                Expr::Identifier(s) => s.clone(),
                // Should never happen because the parser should have caught this
                _ => return (None, Some(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string() }))
            };
            let mut largs = Vec::new(); // `largs` stand for lowered args
            // Iterate over args
            for arg in &args.0 {
                // Lower each argument, if there is an error then return early
                let arg = expr_to_ir(&arg.0);
                if let Some(err) = arg.1 {
                    return (None, Some(err));
                } else {
                    // Else push the lowered argument
                    largs.push(arg.0.unwrap());
                }
            }
            let ir_kind = IRKind::Call { name, args: largs };
            return (Some(ir_kind), None);
        },
        Expr::Fun { name, type_hint, args, body } => {
            // Iterate each argument and give it a type hint
            let args = args.0.iter().map(|arg| (arg.0.0.clone(), gen_type_hint(&arg.1.0))).collect::<Vec<_>>();
            let body = expr_to_ir(&body.0);
            if let Some(err) = body.1 {
                return (None, Some(err));
            } else {
                let body = body.0.unwrap();
                let ir_kind = IRKind::Fun { name: name.clone(), return_type_hint: gen_type_hint(type_hint), args, body: Box::new(body) };
                return (Some(ir_kind), None);
            }
        },
        Expr::Return { expr } => {
            let expr = expr_to_ir(&expr.0);
            if let Some(err) = expr.1 {
                return (None, Some(err));
            } else {
                let expr = expr.0.unwrap();
                let ir_kind = IRKind::Return { value: Box::new(expr) };
                return (Some(ir_kind), None);
            }
        },
        Expr::Do { body } => {
            let mut lbody = Vec::new();
            for expr in body {
                let expr = expr_to_ir(&expr.0);
                if let Some(err) = expr.1 {
                    return (None, Some(err));
                } else {
                    lbody.push(expr.0.unwrap());
                }
            }
            let ir_kind = IRKind::Do { body: lbody };
            return (Some(ir_kind), None);
        },

        // TODO: Handle primitive types error (e.g. overflow)
        // For now it just leaves the value as is and let the target compiler handle it
        Expr::Int(value)        => (Some(IRKind::Value { value: Value::Int(*value) }), None),
        Expr::Boolean(value)    => (Some(IRKind::Value { value: Value::Boolean(*value) }), None),
        Expr::String(value)     => (Some(IRKind::Value { value: Value::String(value.clone()) }), None),
        Expr::Identifier(value) => (Some(IRKind::Value { value: Value::Ident(value.clone()) }), None),
        _ => { dbg!(expr); todo!() }
    }
}

fn gen_type_hint(type_hint: &str) -> String {
    match type_hint {
        "int"    => "int".to_string(),
        "bool"   => "bool".to_string(),
        "string" => "std::string".to_string(),
        _ => { dbg!(type_hint); todo!() }
    }
}
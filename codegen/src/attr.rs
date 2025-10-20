use syn::{parse::{Parse, ParseStream}, Expr, ExprField, Ident, LitStr, Member, ExprPath, Token};
use quote::format_ident;

pub struct ModuleAttr {
    pub ports: Vec<PortDefine>,
}

// address: ("addr{address_width}", Input),
pub struct PortDefine {
    pub name: Ident,              // input / output
    pub pattern: String,          // "A{n}"
    pub direction: Ident,         // Input / Output / InOut
    pub condition: Option<Expr> // e.g. "column_sel_size > 1"
}

impl Parse for ModuleAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ports = Vec::new();
        while !input.is_empty() {
            let name: Ident = input.parse()?;
            let _: Token![:] = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            let pattern: LitStr = content.parse()?;
            let _: Token![,] = content.parse()?;
            let direction: Ident = content.parse()?;

            let condition = if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
                let cond_str: LitStr = content.parse()?;
                let mut cond_expr: Expr = syn::parse_str(&cond_str.value())?;
                rewrite_condition_expr(&mut cond_expr);
                Some(cond_expr)
            } else {
                None
            };

            ports.push(PortDefine { name, pattern: pattern.value(), direction , condition});

            // optional trailing comma
            let _ = input.parse::<Token![,]>();
        }
        Ok(ModuleAttr { ports })
    }
}

fn rewrite_condition_expr(expr: &mut Expr) {
    match expr {
        Expr::Binary(e) => {
            rewrite_condition_expr(&mut *e.left);
            rewrite_condition_expr(&mut *e.right);
        }
        Expr::Unary(e) => rewrite_condition_expr(&mut *e.expr),
        Expr::Paren(e) => rewrite_condition_expr(&mut *e.expr),
        Expr::Group(e) => rewrite_condition_expr(&mut *e.expr),
        Expr::Path(expr_path) => {
            if expr_path.qself.is_none() && expr_path.path.segments.len() == 1 {
                let ident = expr_path.path.segments[0].ident.clone();
                let name = ident.to_string();

                if !matches!(name.as_str(), "module" | "self" | "crate" | "super") {
                    let module_ident = format_ident!("module");
                    let args_ident = format_ident!("args");

                    // 构造 module.args.column_sel_size
                    let new_expr = Expr::Field(ExprField {
                        attrs: Vec::new(),
                        dot_token: Default::default(),
                        member: Member::Named(ident),
                        base: Box::new(Expr::Field(ExprField {
                            attrs: Vec::new(),
                            dot_token: Default::default(),
                            member: Member::Named(args_ident),
                            base: Box::new(Expr::Path(ExprPath {
                                attrs: Vec::new(),
                                qself: None,
                                path: module_ident.into(),
                                
                            })),
                        })),
                    });

                    *expr = new_expr;
                }
            }
        }
        Expr::Call(e) => {
            rewrite_condition_expr(&mut *e.func);
            for a in e.args.iter_mut() {
                rewrite_condition_expr(a);
            }
        }
        Expr::MethodCall(e) => {
            rewrite_condition_expr(&mut *e.receiver);
            for a in e.args.iter_mut() {
                rewrite_condition_expr(a);
            }
        }
        Expr::Index(e) => {
            rewrite_condition_expr(&mut *e.expr);
            rewrite_condition_expr(&mut *e.index);
        }
        Expr::Field(e) => rewrite_condition_expr(&mut *e.base),
        Expr::Assign(e) => {
            rewrite_condition_expr(&mut *e.left);
            rewrite_condition_expr(&mut *e.right);
        }
        Expr::Range(e) => {
            if let Some(from) = &mut e.start {
                rewrite_condition_expr(from);
            }
            if let Some(to) = &mut e.end {
                rewrite_condition_expr(to);
            }
        }
        _ => {}
    }
}

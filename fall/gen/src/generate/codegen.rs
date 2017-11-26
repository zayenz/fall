use serde_json;
use tera::Context;

use fall_tree::{Text, AstNode};
use lang_fall::{FallFile, RefKind, SynRule, LexRule, Expr, BlockExpr, PratVariant, PrattOp,
                CallKind, MethodDef, MethodDescription, Arity, ChildKind, Parameter,
                Analysis};

use fall_parse as dst;

use util::{scream, camel};



pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub(super) struct Codegen<'a, 'f: 'a> {
    analysis: &'a Analysis<'f>,
    node_types: Vec<(Text<'f>, bool)>,
}

impl<'a, 'f> Codegen<'a, 'f> {
    pub fn new(analysis: &'a Analysis<'f>) -> Codegen<'a, 'f> {
        let node_types = {
            let mut result = Vec::new();
            if let Some(tokenizer) = analysis.ast().tokenizer_def() {
                result.extend(
                    tokenizer.lex_rules()
                        .map(|r| (r.node_type(), r.is_skip()))
                )
            }
            result.extend(
                analysis.ast()
                    .syn_rules()
                    .filter(|r| r.is_pub() && r.type_attr().is_none())
                    .filter_map(|r| r.name())
                    .map(|n| (n, false))
            );
            result
        };

        Codegen {
            analysis,
            node_types
        }
    }

    pub fn generate(&mut self) -> Result<Context> {
        let mut context = Context::new();
        context.add("node_types", &self.node_types);

        let mut parser = Vec::new();
        for r in self.file().syn_rules() {
            parser.push(self.gen_rule(r)?)
        }
        let parser = serde_json::to_string(&parser).unwrap();
        context.add("parser_json", &parser);

        let lex_rules = self.file().tokenizer_def()
            .ok_or(format_err!("no tokens defined"))?
            .lex_rules()
            .filter(|r| !r.is_contextual())
            .map(|r| {
                let re = r.token_re().ok_or(format_err!("Bad token"))?;
                Ok(CtxLexRule { ty: r.node_type(), re: format!("{:?}", re), f: r.extern_fn() })
            }).collect::<Result<Vec<_>>>()?;

        context.add("lex_rules", &lex_rules);

        let verbatim = self.file().verbatim_def().map(|v| v.contents());
        context.add("verbatim", &verbatim);
        context.add("has_whitespace_binder", &verbatim.map(|t| t.contains("whitespace_binder")).unwrap_or(false));

        if let Some(ast) = self.file().ast_def() {
            context.add("ast_nodes", &ast.ast_nodes().map(|node| {
                Ok(CtxAstNode {
                    struct_name: camel(node.name()),
                    node_type_name: scream(node.name()),
                    methods: node.methods()
                        .map(|method| generate_method(method))
                        .collect::<Result<Vec<CtxMethod>>>()?
                })
            }).collect::<Result<Vec<_>>>()?);

            context.add("ast_classes", &ast.ast_classes().map(|class| {
                CtxAstClass {
                    enum_name: camel(class.name()),
                    variants: class.variants().map(|variant| (scream(variant), camel(variant))).collect(),
                }
            }).collect::<Vec<_>>());
        }

        Ok(context)
    }

    fn file(&self) -> FallFile<'f> {
        self.analysis.ast()
    }

    fn syn_rule_ty(&self, rule: SynRule<'f>) -> Option<dst::NodeTypeRef> {
        let name = rule.ty_name()?;
        self.node_types.iter()
            .position(|&(ty_name, _)| ty_name == name)
            .map(|i| dst::NodeTypeRef((i + 1) as u32))
    }

    fn syn_rule_ref(&self, rule: SynRule<'f>) -> usize {
        self.file().syn_rules().position(|r| r.node() == rule.node()).unwrap()
    }

    fn lex_rule_ty(&self, rule: LexRule<'f>) -> dst::NodeTypeRef {
        let name = rule.node_type();
        let i = self.node_types.iter()
            .position(|&(ty_name, _)| ty_name == name)
            .unwrap();
        dst::NodeTypeRef((i + 1) as u32)
    }

    fn param_ref(&self, param: Parameter<'f>) -> dst::Arg {
        let idx = self.file().syn_rules()
            .filter_map(|rule| rule.parameters())
            .flat_map(|p| p.parameters())
            .position(|p| p.node() == param.node())
            .unwrap();

        dst::Arg(idx as u32)
    }

    fn gen_rule(&mut self, rule: SynRule<'f>) -> Result<dst::SynRule> {
        let expr = match (rule.is_pratt(), rule.body()) {
            (true, Expr::BlockExpr(block)) => dst::Expr::Pratt(
                Box::new(self.gen_pratt(block)?)
            ),
            (true, _) => unreachable!(),
            (false, body) => self.gen_expr(body)?
        };

        let body = match (self.syn_rule_ty(rule), rule.is_replaces()) {
            (Some(ty_idx), true) => dst::Expr::PubReplace {
                ty_idx,
                body: Box::new(expr),
            },
            (Some(ty_idx), false) => dst::Expr::Pub {
                ty_idx,
                body: Box::new(expr),
                replaceable: rule.is_replaceable(),
            },
            (None, _) => expr,
        };

        Ok(dst::SynRule { body })
    }

    fn gen_expr(&mut self, expr: Expr<'f>) -> Result<dst::Expr> {
        let result = match expr {
            Expr::BlockExpr(block) =>
                dst::Expr::Or(block.alts().map(|e| self.gen_expr(e)).collect::<Result<Vec<_>>>()?),

            Expr::SeqExpr(seq) => {
                fn is_commit(part: Expr) -> bool {
                    part.node().text() == "<commit>"
                }
                let commit = seq.parts().position(is_commit);
                let parts = seq.parts()
                    .filter(|&p| !is_commit(p))
                    .map(|e| self.gen_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                dst::Expr::And(parts, commit)
            }

            Expr::RefExpr(ref_) => {
                let ref_ = self.analysis.resolve_reference(ref_)
                    .ok_or(format_err!("Unresolved references: {}", ref_.node().text()))?;

                match ref_ {
                    RefKind::Token(rule) => {
                        let ty_ref = self.lex_rule_ty(rule);
                        if rule.is_contextual() {
                            dst::Expr::ContextualToken(
                                ty_ref,
                                rule.token_text()
                                    .ok_or(format_err!("Missing contextual token text"))?
                                    .to_string()
                            )
                        } else {
                            dst::Expr::Token(ty_ref)
                        }
                    }
                    RefKind::RuleReference(rule) => dst::Expr::Rule(self.syn_rule_ref(rule)),
                    RefKind::Param(p) => dst::Expr::Var(self.param_ref(p)),
                }
            }

            Expr::CallExpr(call) => {
                let call = self.analysis.resolve_call(call)
                    .ok_or(format_err!("Failed to compile {}", call.node().text()))?;

                let r = match call {
                    CallKind::Eof => dst::Expr::Eof,
                    CallKind::Any => dst::Expr::Any,
                    CallKind::Enter(idx, expr) => dst::Expr::Enter(
                        dst::Context(idx as u32),
                        Box::new(self.gen_expr(expr)?)
                    ),
                    CallKind::Exit(idx, expr) => dst::Expr::Exit(
                        dst::Context(idx as u32),
                        Box::new(self.gen_expr(expr)?)
                    ),
                    CallKind::IsIn(idx) => dst::Expr::IsIn(
                        dst::Context(idx as u32)
                    ),
                    CallKind::Not(expr) => dst::Expr::Not(Box::new(self.gen_expr(expr)?)),
                    CallKind::Layer(e1, e2) => dst::Expr::Layer(
                        Box::new(self.gen_expr(e1)?),
                        Box::new(self.gen_expr(e2)?)
                    ),
                    CallKind::WithSkip(e1, e2) => dst::Expr::WithSkip(
                        Box::new(self.gen_expr(e1)?),
                        Box::new(self.gen_expr(e2)?)
                    ),
                    CallKind::Inject(e1, e2) => dst::Expr::Inject(
                        Box::new(self.gen_expr(e1)?),
                        Box::new(self.gen_expr(e2)?)
                    ),
                    CallKind::RuleCall(rule, args) => dst::Expr::Call(
                        Box::new(dst::Expr::Rule(self.syn_rule_ref(rule))),
                        args.iter()
                            .map(|&(p, e)| Ok((self.param_ref(p), self.gen_expr(e)?)))
                            .collect::<Result<Vec<_>>>()?
                    ),
                    CallKind::PrevIs(tokens) => dst::Expr::PrevIs(
                        tokens.iter().map(|&r| self.syn_rule_ty(r).unwrap()).collect()
                    ),
                    CallKind::Commit => panic!("Should be handled specially"),
                };
                return Ok(r);
            }
            Expr::OptExpr(opt_expr) => dst::Expr::Opt(Box::new(self.gen_expr(opt_expr.expr())?)),
            Expr::RepExpr(rep_expr) => dst::Expr::Rep(Box::new(self.gen_expr(rep_expr.expr())?)),
        };

        Ok(result)
    }

    fn gen_pratt(&mut self, ast: BlockExpr<'f>) -> Result<dst::PrattTable> {
        fn alt_to_rule<'f>(analysis: &Analysis<'f>, alt: Expr<'f>) -> Result<SynRule<'f>> {
            match alt {
                Expr::SeqExpr(expr) => match expr.parts().next() {
                    Some(Expr::RefExpr(ref_)) => match analysis.resolve_reference(ref_) {
                        Some(RefKind::RuleReference(rule)) => Ok(rule),
                        _ => return Err(format_err!("Bad pratt spec")),
                    },
                    _ => return Err(format_err!("Bad pratt spec"))
                },
                _ => return Err(format_err!("Bad pratt spec"))
            }
        }

        let mut result = dst::PrattTable {
            atoms: Vec::new(),
            prefixes: Vec::new(),
            infixes: Vec::new(),
        };
        for alt in ast.alts() {
            let rule = alt_to_rule(&self.analysis, alt)?;

            let ty = self.syn_rule_ty(rule)
                .ok_or(format_err!("non public pratt rule"))?;

            let prat_kind = self.analysis.resolve_pratt_variant(rule)
                .ok_or(format_err!("pratt rule without attributes"))?;

            match prat_kind {
                PratVariant::Atom(_) =>
                    result.atoms.push(self.gen_rule(rule)?.body),
                PratVariant::Postfix(PrattOp { op, priority }) => {
                    result.infixes.push(dst::Infix {
                        ty,
                        op: self.gen_expr(op)?,
                        priority,
                        has_rhs: false,
                    });
                }
                PratVariant::Prefix(PrattOp { op, priority }) => {
                    result.prefixes.push(dst::Prefix {
                        ty,
                        op: self.gen_expr(op)?,
                        priority,
                    })
                }
                PratVariant::Bin(PrattOp { op, priority }) => {
                    result.infixes.push(dst::Infix {
                        ty,
                        op: self.gen_expr(op)?,
                        priority,
                        has_rhs: true,
                    });
                }
            };
        }

        Ok(result)
    }
}

fn generate_method<'f>(method: MethodDef<'f>) -> Result<CtxMethod<'f>> {
    let description = method.resolve()
        .ok_or(format_err!("Bad method `{}`", method.node().text()))?;

    let (ret_type, body) = match description {
        MethodDescription::TextAccessor(lex_rule, arity) => {
            let node_type = scream(lex_rule.node_type());
            match arity {
                Arity::Single =>
                    ("Text<'f>".to_owned(),
                     format!("child_of_type_exn(self.node, {}).text()", node_type)),

                Arity::Optional =>
                    ("Option<Text<'f>>".to_owned(),
                     format!("child_of_type(self.node, {}).map(|n| n.text())", node_type)),

                Arity::Many => unimplemented!(),
            }
        }
        MethodDescription::NodeAccessor(kind, arity) => {
            match (kind, arity) {
                (ChildKind::AstNode(n), Arity::Single) =>
                    (format!("{}<'f>", camel(n.name())),
                     "AstChildren::new(self.node.children()).next().unwrap()".to_owned()),
                (ChildKind::AstNode(n), Arity::Optional) =>
                    (format!("Option<{}<'f>>", camel(n.name())),
                     "AstChildren::new(self.node.children()).next()".to_owned()),
                (ChildKind::AstNode(n), Arity::Many) =>
                    (format!("AstChildren<'f, {}<'f>>", camel(n.name())),
                     "AstChildren::new(self.node.children())".to_owned()),

                (ChildKind::AstClass(n), Arity::Single) =>
                    (format!("{}<'f>", camel(n.name())),
                     "AstChildren::new(self.node.children()).next().unwrap()".to_owned()),
                (ChildKind::AstClass(n), Arity::Optional) =>
                    (format!("Option<{}<'f>>", camel(n.name())),
                     "AstChildren::new(self.node.children()).next()".to_owned()),
                (ChildKind::AstClass(n), Arity::Many) =>
                    (format!("AstChildren<'f, {}<'f>>", camel(n.name())),
                     "AstChildren::new(self.node.children())".to_owned()),

                (ChildKind::Token(lex_rule), arity) => {
                    let node_type = scream(lex_rule.node_type());
                    match arity {
                        Arity::Single =>
                            ("Node<'f>".to_owned(),
                             format!("self.node().children().find(|n| n.ty() == {}).unwrap()", node_type)),
                        Arity::Optional =>
                            ("Option<Node<'f>>".to_owned(),
                             format!("self.node().children().find(|n| n.ty() == {})", node_type)),
                        Arity::Many => unimplemented!(),
                    }
                }
            }
        }
    };

    Ok(CtxMethod { name: method.name(), ret_type, body })
}

#[derive(Serialize)]
struct CtxLexRule<'f> {
    ty: Text<'f>,
    re: String,
    f: Option<Text<'f>>
}

#[derive(Serialize)]
struct CtxAstNode<'f> {
    struct_name: String,
    node_type_name: String,
    methods: Vec<CtxMethod<'f>>
}

#[derive(Serialize)]
struct CtxAstClass {
    enum_name: String,
    variants: Vec<(String, String)>
}

#[derive(Serialize)]
struct CtxMethod<'f> {
    name: Text<'f>,
    ret_type: String,
    body: String
}

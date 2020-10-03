use super::{PratVariant, PrattOp};
use crate::analysis::db::{self, DB};
use crate::analysis::diagnostics::DiagnosticSink;
use crate::syntax::Expr;
use fall_tree::AstNode;
use itertools::Itertools;

impl<'f> db::OnceQExecutor<'f> for super::ResolvePrattVariant<'f> {
    fn execute(self, _: &DB<'f>, d: &mut DiagnosticSink) -> Option<PratVariant<'f>> {
        let rule = self.0;
        let name_ident = match rule.name_ident() {
            None => return None,
            Some(n) => n,
        };
        let kinds = ["atom", "postfix", "prefix", "bin"];

        let (kind, priority) = {
            let attrs = match rule.attributes() {
                None => return None,
                Some(attrs) => attrs,
            };
            match kinds
                .iter()
                .filter_map(|&k| attrs.find(k).map(|a| (k, a)))
                .next()
                .map(|(k, a)| (k, a.u32_value()))
            {
                Some(a) => a,
                None => return None,
            }
        };

        if kind == "atom" {
            if priority.is_some() {
                d.error(name_ident, "Atom rules don't have priority")
            }
            return Some(PratVariant::Atom(rule.body()));
        }

        let args = match rule.body() {
            Expr::BlockExpr(block) => match block.alts().collect_tuple() {
                Some((alt,)) => match alt {
                    Expr::SeqExpr(args) => args.parts(),
                    _ => return None,
                },
                None => {
                    d.error(
                        block.node(),
                        "Expression rule requires a single alternative",
                    );
                    return None;
                }
            },
            _ => return None,
        };

        let result = match kind {
            "postfix" => {
                if let Some((_expr, op)) = args.collect_tuple() {
                    PratVariant::Postfix(PrattOp {
                        op,
                        priority: priority.unwrap_or(999),
                    })
                } else {
                    d.error(
                        rule.body().node(),
                        "Postfix rule requires a single expression and an operation",
                    );
                    return None;
                }
            }
            "prefix" => {
                if let Some((op, _expr)) = args.collect_tuple() {
                    PratVariant::Prefix(PrattOp {
                        op,
                        priority: priority.unwrap_or(999),
                    })
                } else {
                    d.error(
                        rule.body().node(),
                        "Prefix rule requires an operation and a single expression",
                    );
                    return None;
                }
            }
            "bin" => match (args.collect_tuple(), priority) {
                (Some((_lhs, op, _rhs)), Some(priority)) => {
                    PratVariant::Bin(PrattOp { op, priority })
                }
                (None, _) => {
                    d.error(
                        rule.body().node(),
                        "Binary expression requires a left hand side expression, an operator \
                             and a right hand side expression",
                    );
                    return None;
                }
                (Some(_), None) => {
                    d.error(name_ident, "Binary expression requires explicit priority");
                    return None;
                }
            },
            _ => unreachable!(),
        };
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::*;

    #[test]
    fn pratt_atom() {
        check_diagnostics(
            "#[atom(92)] rule foo { }",
            "E foo: Atom rules don't have priority",
        );
    }

    #[test]
    fn pratt_args() {
        check_diagnostics(
            "#[postfix] rule foo { foo | foo }",
            "E { foo | foo }: Expression rule requires a single alternative",
        );
    }

    #[test]
    fn pratt_postfix() {
        check_diagnostics(
            "#[postfix] rule foo { <eof> <eof> <eof>}",
            "E { <eof> <eof> <eof>}: Postfix rule requires a single expression and an operation",
        );
    }

    #[test]
    fn pratt_prefix() {
        check_diagnostics(
            "#[prefix] rule foo { }",
            "E { }: Prefix rule requires an operation and a single expression",
        );
    }

    #[test]
    fn pratt_bin() {
        check_diagnostics(
            "#[bin(92)] rule foo { }",
            "E { }: Binary expression requires a left hand side expression, an operator \
             and a right hand side expression",
        );
        check_diagnostics(
            "#[bin] rule foo { <eof> <eof> <eof> }",
            "E foo: Binary expression requires explicit priority",
        );
    }
}

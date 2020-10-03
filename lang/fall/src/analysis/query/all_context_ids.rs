use std::collections::btree_map::{self, BTreeMap};
use std::sync::Arc;

use crate::analysis::db::{self, DB};
use crate::analysis::diagnostics::DiagnosticSink;
use crate::syntax::CallExpr;
use fall_tree::visitor::{process_subtree_bottom_up, visitor};
use fall_tree::{AstNode, Text};

impl<'f> db::OnceQExecutor<'f> for super::AllContexts {
    fn execute(self, db: &DB<'f>, d: &mut DiagnosticSink) -> Arc<Vec<Text<'f>>> {
        let result = process_subtree_bottom_up(
            db.file().node(),
            visitor(BTreeMap::<Text<'f>, Option<CallExpr<'f>>>::new()).visit::<CallExpr, _>(
                |call, contexts| {
                    if let Some(ctx) = call.context_name() {
                        match contexts.entry(ctx) {
                            btree_map::Entry::Occupied(mut occupied) => {
                                occupied.insert(None);
                            }
                            btree_map::Entry::Vacant(vacant) => {
                                vacant.insert(Some(call));
                            }
                        }
                    }
                },
            ),
        );

        for (k, v) in result.iter() {
            if let Some(call) = v {
                d.warning(call.node(), format!("Context `{}` is used only once", k))
            };
        }

        Arc::new(result.into_iter().map(|(k, _)| k).collect())
    }
}

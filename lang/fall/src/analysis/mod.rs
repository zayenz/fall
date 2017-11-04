use std::collections::HashSet;
use std::sync::Mutex;

use fall_tree::{File, Node, AstNode, Text};
use fall_tree::visitor::{Visitor, NodeVisitor};
use fall_tree::search::child_of_type;
use fall_tree::search::ast;

use {FallFile, SynRule, RefExpr, CallExpr, SYN_RULE};
use editor_api::{Diagnostic, Severity};

mod calls;
mod references;
mod diagnostics;
mod cache;

use self::diagnostics::DiagnosticSink;
use self::cache::{FileCache, NodeCache};
pub use self::calls::CallKind;
pub use self::references::RefKind;


pub struct Analysis<'f> {
    file: FallFile<'f>,

    diagnostics: Mutex<Vec<Diagnostic>>,

    used_rules: FileCache<HashSet<Node<'f>>>,
    contexts: FileCache<Vec<Text<'f>>>,

    calls: NodeCache<'f, Option<CallKind<'f>>>,
    refs: NodeCache<'f, Option<RefKind<'f>>>,
}

impl<'f> Analysis<'f> {
    pub fn new(file: FallFile) -> Analysis {
        Analysis {
            file,
            diagnostics: Default::default(),
            used_rules: Default::default(),
            contexts: Default::default(),
            calls: Default::default(),
            refs: Default::default(),
        }
    }

    pub fn file(&self) -> FallFile<'f> {
        self.file
    }

    pub fn resolve_call(&self, call: CallExpr<'f>) -> Option<CallKind<'f>> {
        let mut diagnostics = Vec::new();
        let (value, committed) = {
            let mut sink = DiagnosticSink::new(&mut diagnostics);
            self.calls.get(call.node(), || {
                calls::resolve(self, &mut sink, call)
            })
        };
        if committed {
            self.diagnostics.lock().unwrap().extend(diagnostics)
        }
        value
    }

    pub fn resolve_reference(&self, ref_: RefExpr<'f>) -> Option<RefKind<'f>> {
        let mut diagnostics = Vec::new();
        let (value, committed) = {
            let mut sink = DiagnosticSink::new(&mut diagnostics);
            self.refs.get(ref_.node(), || {
                references::resolve(self, &mut sink, ref_)
            })
        };
        if committed {
            self.diagnostics.lock().unwrap().extend(diagnostics)
        }
        value
    }

    pub fn collect_all_diagnostics(&self) -> Vec<Diagnostic> {
        let mut result = Visitor(Vec::new())
            .visit::<RefExpr, _>(|_, ref_| { self.resolve_reference(ref_); })
            .visit::<CallExpr, _>(|_, call| { self.resolve_call(call); })
            .visit::<SynRule, _>(|acc, rule| {
                if !self.used_rules().contains(&rule.node()) {
                    if let Some(rule_name) = rule.name_ident() {
                        acc.push(Diagnostic {
                            range: rule_name.range(),
                            severity: Severity::Warning,
                            message: "Unused rule".to_string(),
                        })
                    }
                }
            })
            .walk_recursively_children_first(self.file().node());

        result.extend(self.diagnostics());
        result.sort_by_key(|d| (d.range.start(), d.range.end()));
        result
    }

    #[allow(unused)] // for debugging
    pub fn debug_diagnostics(&self) -> String {
        diagnostics::debug_diagnostics(&self.diagnostics(), self.file().node().text())
    }

    fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.lock().unwrap().clone()
    }

    fn contexts(&self) -> &[Text<'f>] {
        self.contexts.get(|| calls::contexts(self)).as_ref()
    }

    fn rule_by_name(&self, reference_name: Text<'f>) -> Option<SynRule<'f>> {
        self.file().syn_rules().find(|r| r.name() == Some(reference_name))
    }

    fn used_rules(&self) -> &HashSet<Node<'f>> {
        self.used_rules.get(|| self.calculate_used_rules())
    }

    fn calculate_used_rules(&self) -> HashSet<Node<'f>> {
        ast::descendants_of_type::<RefExpr>(self.file.node())
            .into_iter()
            .filter_map(|ref_| self.resolve_reference(ref_))
            .filter_map(|r| match r {
                RefKind::RuleReference(rule) => Some(rule.node()),
                _ => None
            })
            .chain(
                ast::descendants_of_type::<CallExpr>(self.file.node())
                    .into_iter()
                    .filter_map(|call| self.resolve_call(call))
                    .filter_map(|kind| match kind {
                        CallKind::RuleCall(rule, ..) => Some(rule.node()),
                        _ => None,
                    })
            )
            .chain(child_of_type(self.file.node(), SYN_RULE).into_iter())
            .collect()
    }
}

pub struct FileWithAnalysis {
    rent: rent::R
}

impl FileWithAnalysis {
    pub fn new(text: String) -> FileWithAnalysis {
        FileWithAnalysis::from_file(::parse(text))
    }

    fn from_file(file: File) -> FileWithAnalysis {
        FileWithAnalysis {
            rent: rent::R::new(Box::new(file), |file| {
                Analysis::new(FallFile::new(file.root()))
            })
        }
    }

    pub fn file(&self) -> &File {
        self.rent.head()
    }

    pub fn analyse<T, F: FnOnce(&Analysis) -> T>(&self, f: F) -> T {
        self.rent.rent(|a: &Analysis| f(a))
    }
}

rental! {
    mod rent {
        use super::*;

        #[rental]
        pub struct R {
            file: Box<File>,
            analysis: Analysis<'file>
        }
    }
}

#[test]
fn analysis_is_sync() {
    fn assert_is_sync<T: Sync>() {}
    assert_is_sync::<Analysis>();
}

#[cfg(test)]
fn check_diagnostics(code: &str, expected_diagnostics: &str) {
    use fall_tree::test_util::report_diff;
    let file = ::editor_api::analyse(code.to_string());

    file.analyse(|a| {
        let d = a.collect_all_diagnostics();
        let actual = d.into_iter().map(|d| {
            let s = match d.severity {
                Severity::Error => 'E',
                Severity::Warning => 'W',
            };
            format!("{}: {} {}", a.file().node().text().slice(d.range), s, d.message)
        }).collect::<Vec<_>>().join("\n");

        report_diff(&actual, expected_diagnostics);
    })
}

#[test]
fn test_diagnostics() {
    check_diagnostics(r"
       pub rule foo { <eof x> }
       rule bar { foo <abracadabra>}
       rule baz { <prev_is foo> <prev_is bar> <prev_is {foo}>}
    ", "\
<eof x>: E Wrong number of arguments, expected 0, got 1
x: E Unresolved reference
abracadabra: E Unresolved reference
baz: W Unused rule
<prev_is bar>: E <prev_is> arguments must be public rules
<prev_is {foo}>: E <prev_is> arguments must be public rules");
}

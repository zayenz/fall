use fall_editor::hl::{self, Highlights, HlTag};
use fall_tree::search::child_of_type;
use fall_tree::visitor::process_subtree_bottom_up;
use fall_tree::{AstNode, Node, NodeType};

use crate::analysis::{Analysis, CallKind, ChildKind, MethodKind, RefKind};
use crate::syntax::*;

pub(crate) fn highlight(analysis: &Analysis) -> Highlights {
    process_subtree_bottom_up(
        analysis.ast().node(),
        hl::visitor(&[
            (hl::COMMENT, &[EOL_COMMENT]),
            (hl::STRING, &[HASH_STRING, SIMPLE_STRING]),
            (
                hl::KEYWORD,
                &[
                    RULE, VERBATIM, TOKENIZER, AST, NODE, CLASS, PUB, TEST, TRAIT,
                ],
            ),
            (hl::PARAMETER, &[PARAMETER]),
            (hl::ATTRIBUTE, &[ATTRIBUTES]),
        ])
        .visit::<LexRule, _>(|rule, hls| colorize_child(rule.node(), IDENT, hl::LITERAL, hls))
        .visit::<SynRule, _>(|rule, hls| colorize_child(rule.node(), IDENT, hl::FUNCTION, hls))
        .visit::<AstNodeDef, _>(|rule, hls| colorize_child(rule.node(), IDENT, hl::FUNCTION, hls))
        .visit::<AstTraitDef, _>(|rule, hls| colorize_child(rule.node(), IDENT, hl::FUNCTION, hls))
        .visit::<AstClassDef, _>(|rule, hls| colorize_child(rule.node(), IDENT, hl::FUNCTION, hls))
        .visit::<RefExpr, _>(|ref_, hls| {
            let color = match analysis.resolve_reference(ref_) {
                Some(RefKind::Token(_)) => hl::LITERAL,
                Some(RefKind::RuleReference { .. }) => hl::FUNCTION,
                Some(RefKind::Param(..)) => hl::PARAMETER,
                None => return,
            };
            colorize_node(ref_.node(), color, hls)
        })
        .visit::<MethodDef, _>(|method, hls| {
            let color = match analysis.resolve_method(method) {
                Some(MethodKind::NodeAccessor(child_kind, _)) => match child_kind {
                    ChildKind::Token(..) => hl::LITERAL,
                    ChildKind::AstClass(..) | ChildKind::AstNode(..) => hl::FUNCTION,
                },
                None | Some(_) => return,
            };
            colorize_child(method.selector().node(), IDENT, color, hls)
        })
        .visit::<CallExpr, _>(|call, hls| {
            let color = match analysis.resolve_call(call) {
                None | Some(CallKind::RuleCall(..)) => hl::FUNCTION,
                Some(_) => hl::BUILTIN,
            };

            colorize_child(call.node(), IDENT, color, hls);
            colorize_child(call.node(), L_ANGLE, color, hls);
            colorize_child(call.node(), R_ANGLE, color, hls);
        }),
    )
}

fn colorize_node(node: Node, tag: HlTag, spans: &mut Highlights) {
    spans.push((node.range(), tag))
}

fn colorize_child(node: Node, child: NodeType, tag: HlTag, spans: &mut Highlights) {
    if let Some(child) = child_of_type(node, child) {
        colorize_node(child, tag, spans);
    }
}

#[test]
fn test_highlighting() {
    let file = crate::analyse(
        r####"
tokenizer { number r"\d+"}
pub rule foo { bar <eof> <abracadabra> }
rule bar { number <m foo> }

rule m(f) {}
"####
            .to_owned(),
    );

    file.analyse(|a| {
        let spans = highlight(a);
        let result = spans
            .into_iter()
            .map(|(range, d)| format!("{}: {}", a.ast().node().text().slice(range), d.0))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            result,
            r##"tokenizer: keyword
r"\d+": string
number: literal
pub: keyword
rule: keyword
bar: function
eof: builtin
<: builtin
>: builtin
abracadabra: function
<: function
>: function
foo: function
rule: keyword
number: literal
foo: function
m: function
<: function
>: function
bar: function
rule: keyword
f: parameter
m: function"##
        );
    });
}

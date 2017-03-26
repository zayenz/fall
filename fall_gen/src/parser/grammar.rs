use std::sync::{Once, ONCE_INIT};
use fall_tree::{NodeType, NodeTypeInfo};
use fall_parse::Rule;
pub use fall_tree::{ERROR, WHITESPACE};

pub const KW_NODES  : NodeType = NodeType(100);
pub const KW_TOKENIZER: NodeType = NodeType(101);
pub const KW_RULE   : NodeType = NodeType(102);
pub const EQ        : NodeType = NodeType(103);
pub const PIPE      : NodeType = NodeType(104);
pub const START     : NodeType = NodeType(105);
pub const LBRACE    : NodeType = NodeType(106);
pub const RBRACE    : NodeType = NodeType(107);
pub const IDENT     : NodeType = NodeType(108);
pub const SIMPLE_STRING: NodeType = NodeType(109);
pub const HASH_STRING: NodeType = NodeType(110);
pub const FILE      : NodeType = NodeType(111);
pub const STRING    : NodeType = NodeType(112);
pub const NODES_DEF : NodeType = NodeType(113);
pub const TOKENIZER_DEF: NodeType = NodeType(114);
pub const RULE_DEF  : NodeType = NodeType(115);
pub const RULE      : NodeType = NodeType(116);

pub fn register_node_types() {
    static REGISTER: Once = ONCE_INIT;
    REGISTER.call_once(||{
        KW_NODES.register(NodeTypeInfo { name: "KW_NODES" });
        KW_TOKENIZER.register(NodeTypeInfo { name: "KW_TOKENIZER" });
        KW_RULE.register(NodeTypeInfo { name: "KW_RULE" });
        EQ.register(NodeTypeInfo { name: "EQ" });
        PIPE.register(NodeTypeInfo { name: "PIPE" });
        START.register(NodeTypeInfo { name: "START" });
        LBRACE.register(NodeTypeInfo { name: "LBRACE" });
        RBRACE.register(NodeTypeInfo { name: "RBRACE" });
        IDENT.register(NodeTypeInfo { name: "IDENT" });
        SIMPLE_STRING.register(NodeTypeInfo { name: "SIMPLE_STRING" });
        HASH_STRING.register(NodeTypeInfo { name: "HASH_STRING" });
        FILE.register(NodeTypeInfo { name: "FILE" });
        STRING.register(NodeTypeInfo { name: "STRING" });
        NODES_DEF.register(NodeTypeInfo { name: "NODES_DEF" });
        TOKENIZER_DEF.register(NodeTypeInfo { name: "TOKENIZER_DEF" });
        RULE_DEF.register(NodeTypeInfo { name: "RULE_DEF" });
        RULE.register(NodeTypeInfo { name: "RULE" });
    });
}

pub const TOKENIZER: &'static [Rule] = &[
    Rule { ty: WHITESPACE, re: r"\s+", f: None },
    Rule { ty: EQ, re: "=", f: None },
    Rule { ty: PIPE, re: "*", f: None },
    Rule { ty: START, re: "*", f: None },
    Rule { ty: LBRACE, re: r"\{", f: None },
    Rule { ty: RBRACE, re: r"\}", f: None },
    Rule { ty: SIMPLE_STRING, re: r#"r?"([^"\\]|\\.)*""#, f: None },
    Rule { ty: HASH_STRING, re: "r#+", f: Some(super::parse_raw_string) },
    Rule { ty: KW_NODES, re: "nodes", f: None },
    Rule { ty: KW_TOKENIZER, re: "tokenizer", f: None },
    Rule { ty: IDENT, re: r"\w+", f: None },
];
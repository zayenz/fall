use serde_json;
use fall_tree::{NodeType, NodeTypeInfo, Language, LanguageImpl};
pub use fall_tree::{ERROR, WHITESPACE};

pub const LPAREN: NodeType = NodeType(100);
pub const RPAREN: NodeType = NodeType(101);
pub const LBRACE: NodeType = NodeType(102);
pub const RBRACE: NodeType = NodeType(103);
pub const EQ: NodeType = NodeType(104);
pub const SEMI: NodeType = NodeType(105);
pub const KW_PUB: NodeType = NodeType(106);
pub const KW_LET: NodeType = NodeType(107);
pub const STRUCT: NodeType = NodeType(108);
pub const FN: NodeType = NodeType(109);
pub const IDENT: NodeType = NodeType(110);
pub const NUMBER: NodeType = NodeType(111);
pub const FILE: NodeType = NodeType(112);
pub const FN_DEF: NodeType = NodeType(113);
pub const STRUCT_DEF: NodeType = NodeType(114);
pub const BLOCK_EXPR: NodeType = NodeType(115);
pub const STMT: NodeType = NodeType(116);
pub const PATTERN: NodeType = NodeType(117);
pub const EXPR: NodeType = NodeType(118);

lazy_static! {
    pub static ref LANG: Language = {
        use fall_parse::{LexRule, SynRule, Parser};
        const ALL_NODE_TYPES: &[NodeType] = &[
            ERROR, WHITESPACE,
            LPAREN, RPAREN, LBRACE, RBRACE, EQ, SEMI, KW_PUB, KW_LET, STRUCT, FN, IDENT, NUMBER, FILE, FN_DEF, STRUCT_DEF, BLOCK_EXPR, STMT, PATTERN, EXPR,
        ];
        let parser_json = r##"[{"ty":14,"body":{"Or":[{"And":[[{"Rep":{"Or":[{"And":[[{"SkipUntil":[8,11,10]},{"Rule":1}],null]}]}}],null]}]}},{"ty":null,"body":{"Or":[{"And":[[{"Rule":2}],null]},{"And":[[{"Rule":3}],null]}]}},{"ty":15,"body":{"Or":[{"And":[[{"Opt":{"Or":[{"And":[[{"Token":8}],null]}]}},{"Token":11},{"Token":12},{"Token":2},{"Token":3},{"Rule":4}],2]}]}},{"ty":16,"body":{"Or":[{"And":[[{"Opt":{"Or":[{"And":[[{"Token":8}],null]}]}},{"Token":10},{"Token":12},{"Token":4},{"Token":5}],2]}]}},{"ty":17,"body":{"Or":[{"And":[[{"Token":4},{"Rep":{"Rule":5}},{"Token":5}],null]}]}},{"ty":18,"body":{"Or":[{"And":[[{"Token":9},{"Rule":6},{"Token":6},{"Rule":7},{"Token":7}],1]}]}},{"ty":19,"body":{"Or":[{"And":[[{"Token":12}],null]}]}},{"ty":20,"body":{"Or":[{"And":[[{"Token":13}],null]}]}}]"##;
        let parser: Vec<SynRule> = serde_json::from_str(parser_json).unwrap();

        struct Impl { tokenizer: Vec<LexRule>, parser: Vec<SynRule> };
        impl LanguageImpl for Impl {
            fn parse(&self, lang: Language, text: String) -> ::fall_tree::File {
                ::fall_parse::parse(lang, text, &self.tokenizer, &|tokens, stats| {
                    Parser::new(ALL_NODE_TYPES, &self.parser).parse(tokens, stats)
                })
            }

            fn node_type_info(&self, ty: NodeType) -> NodeTypeInfo {
                match ty {
                    ERROR => NodeTypeInfo { name: "ERROR" },
                    WHITESPACE => NodeTypeInfo { name: "WHITESPACE" },
                    LPAREN => NodeTypeInfo { name: "LPAREN" },
                    RPAREN => NodeTypeInfo { name: "RPAREN" },
                    LBRACE => NodeTypeInfo { name: "LBRACE" },
                    RBRACE => NodeTypeInfo { name: "RBRACE" },
                    EQ => NodeTypeInfo { name: "EQ" },
                    SEMI => NodeTypeInfo { name: "SEMI" },
                    KW_PUB => NodeTypeInfo { name: "KW_PUB" },
                    KW_LET => NodeTypeInfo { name: "KW_LET" },
                    STRUCT => NodeTypeInfo { name: "STRUCT" },
                    FN => NodeTypeInfo { name: "FN" },
                    IDENT => NodeTypeInfo { name: "IDENT" },
                    NUMBER => NodeTypeInfo { name: "NUMBER" },
                    FILE => NodeTypeInfo { name: "FILE" },
                    FN_DEF => NodeTypeInfo { name: "FN_DEF" },
                    STRUCT_DEF => NodeTypeInfo { name: "STRUCT_DEF" },
                    BLOCK_EXPR => NodeTypeInfo { name: "BLOCK_EXPR" },
                    STMT => NodeTypeInfo { name: "STMT" },
                    PATTERN => NodeTypeInfo { name: "PATTERN" },
                    EXPR => NodeTypeInfo { name: "EXPR" },
                    _ => panic!("Unknown NodeType: {:?}", ty)
                }
            }
        }

        Language::new(Impl {
            tokenizer: vec![
                LexRule::new(LPAREN, "\\(", None),
                LexRule::new(RPAREN, "\\)", None),
                LexRule::new(LBRACE, "\\{", None),
                LexRule::new(RBRACE, "\\}", None),
                LexRule::new(EQ, "=", None),
                LexRule::new(SEMI, ";", None),
                LexRule::new(KW_PUB, "pub", None),
                LexRule::new(KW_LET, "let", None),
                LexRule::new(STRUCT, "struct", None),
                LexRule::new(FN, "fn", None),
                LexRule::new(WHITESPACE, "\\s+", None),
                LexRule::new(IDENT, "\\p{XID_Start}\\w*", None),
                LexRule::new(NUMBER, "\\d+", None),
            ],
            parser: parser,
        })
    };
}



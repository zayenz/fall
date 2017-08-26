extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate elapsed;
pub extern crate regex;
pub extern crate fall_tree;
pub extern crate serde_json;

use regex::Regex;
use fall_tree::{Language, NodeType, FileStats, INode};

mod lex_engine;
mod syn_engine;
mod tree_builder;


pub struct ParserDefinition {
    pub node_types: Vec<NodeType>,
    pub lexical_rules: Vec<LexRule>,
    pub syntactical_rules: Vec<SynRule>
}

impl ParserDefinition {
    pub fn parse(&self, text: &str, lang: &Language) -> (FileStats, INode) {
        self::tree_builder::parse(
            text,
            lang,
            &self.lexical_rules,
            &|tokens, stats| {
                let (node, ticks) = ::syn_engine::parse(
                    &self.node_types,
                    &self.syntactical_rules,
                    tokens
                );
                stats.parsing_ticks = ticks;
                node
            }
        )
    }
}

pub struct LexRule {
    pub ty: NodeType,
    pub re: Regex,
    pub f: Option<CustomLexRule>,
}

pub type CustomLexRule = fn(&str) -> Option<usize>;

impl LexRule {
    pub fn new(ty: NodeType, re: &str, f: Option<CustomLexRule>) -> LexRule {
        LexRule {
            ty: ty,
            re: Regex::new(&format!("^({})", re)).unwrap(),
            f: f,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SynRule {
    pub body: Expr,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Expr {
    Pub {
        ty_idx: usize,
        body: Box<Expr>,
        replaceable: bool,
    },
    PubReplace {
        ty_idx: usize,
        body: Box<Expr>,
    },
    Or(Vec<Expr>),
    And(Vec<Expr>, Option<usize>),
    Rule(usize),
    Token(usize),
    ContextualToken(usize, String),
    Rep(Box<Expr>),
    WithSkip(Box<Expr>, Box<Expr>),
    Opt(Box<Expr>),
    Not(Box<Expr>),
    Eof,
    Any,
    Layer(Box<Expr>, Box<Expr>),
    Pratt(Vec<PrattVariant>),
    Enter(u32, Box<Expr>),
    Exit(u32, Box<Expr>),
    IsIn(u32),
    Call(Box<Expr>, Vec<(u32, Expr)>),
    Var(u32),
    PrevIs(Vec<usize>)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PrattVariant {
    Atom { body: Box<Expr> },
    Binary {
        ty: usize,
        op: Box<Expr>,
        priority: u32,
    },
    Postfix {
        ty: usize,
        op: Box<Expr>
    },
    Prefix {
        ty: usize,
        op: Box<Expr>
    }
}

pub mod runtime {
    pub use serde_json;
    pub use regex;
    pub use fall_tree;
    pub use lazy_static::*;
    pub use tree_builder::parse;
}

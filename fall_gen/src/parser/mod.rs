pub mod grammar;

use fall_tree::File;
use fall_parse::{self, TreeBuilder};
use self::grammar::*;

pub fn parse(text: String) -> File {
    register_node_types();
    fall_parse::parse(
        text,
        grammar::FILE,
        grammar::TOKENIZER,
        &parse_file
    )
}

fn parse_file(b: &mut TreeBuilder) {
    parse_nodes(b);
    b.skip_until(&[KW_TOKENIZER]);
    parse_tokenizer(b);
}

fn parse_nodes(b: &mut TreeBuilder) -> bool {
    b.start(NODES_DEF);
    if !b.try_eat(KW_NODES) {
        b.rollback(NODES_DEF);
        return false;
    }
    if b.try_eat(LBRACE) {
        b.parse_many(&|b| b.try_eat(IDENT));
        b.try_eat(RBRACE);
    }
    b.finish(NODES_DEF);
    true
}

fn parse_tokenizer(b: &mut TreeBuilder) -> bool {
    b.start(TOKENIZER_DEF);
    if !b.try_eat(KW_TOKENIZER) {
        b.rollback(TOKENIZER_DEF);
        return false;
    }
    if b.try_eat(LBRACE) {
        b.parse_many(&|b| {
            b.skip_until(&[RBRACE, IDENT]);
            parse_rule(b)
        });
        b.try_eat(RBRACE);
    }
    b.finish(TOKENIZER_DEF);
    true
}

fn parse_rule(b: &mut TreeBuilder) -> bool {
    b.start(RULE);
    if !b.try_eat(IDENT) {
        b.rollback(RULE);
        return false
    }
    parse_string(b) && parse_string(b);
    b.finish(RULE);
    true
}

fn parse_string(b: &mut TreeBuilder) -> bool {
    b.start(STRING);
    if b.try_eat(SIMPLE_STRING) || b.try_eat(HASH_STRING) {
        b.finish(STRING);
        true
    } else {
        b.rollback(STRING);
        false
    }
}

fn parse_raw_string(s: &str) -> Option<usize> {
    let quote_start = s.find('"').unwrap();
    let closing = &"\"########################"[..quote_start];
    s[quote_start + 1..].find(closing).map(|i| i + quote_start + 1 + closing.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn match_ast(actual: &str, expected: &str) {
        let actual = actual.trim();
        let expected = expected.trim();
        if actual != expected {
            panic!("Actual:\n{}\nExpected:\n{}\n", actual, expected)
        }
    }

    fn ast(code: &str) -> String {
        parse(code.to_owned()).dump()
    }

    #[test]
    fn empty_file() {
        match_ast(&ast(""), r#"FILE """#)
    }

    #[test]
    fn nodes() {
        match_ast(&ast("nodes { foo bar }"), r#"
FILE
  NODES_DEF
    KW_NODES "nodes"
    LBRACE "{"
    IDENT "foo"
    IDENT "bar"
    RBRACE "}"
"#)
    }

    #[test]
    fn tokenizer() {
        match_ast(&ast(r#"nodes {} tokenizer { foo "foo" id r"\w+" ext "ext" "super::ext"}"#), r#"
FILE
  NODES_DEF
    KW_NODES "nodes"
    LBRACE "{"
    RBRACE "}"
  TOKENIZER_DEF
    KW_TOKENIZER "tokenizer"
    LBRACE "{"
    RULE
      IDENT "foo"
      STRING
        SIMPLE_STRING "\"foo\""
    RULE
      IDENT "id"
      STRING
        SIMPLE_STRING "r\"\\w+\""
    RULE
      IDENT "ext"
      STRING
        SIMPLE_STRING "\"ext\""
      STRING
        SIMPLE_STRING "\"super::ext\""
    RBRACE "}"
"#) } }
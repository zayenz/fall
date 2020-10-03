use fall_tree::{tu, NodeType, Text, TextEdit, TextEditOp, TextSuffix, TextUnit, ERROR};
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub struct Token {
    pub ty: NodeType,
    pub len: TextUnit,
}

pub trait Lexer {
    fn next_token(&self, text: Text) -> Token;

    fn step(&self, text: &mut Text) -> Token {
        let t = self.next_token(*text);
        *text = text.slice(TextSuffix::from(t.len));
        t
    }
}

pub fn lex<L: Lexer>(lexer: &L, text: Text) -> Vec<Token> {
    let mut result = Vec::new();
    let mut text = text;
    while !text.is_empty() {
        let t = lexer.step(&mut text);
        result.push(t);
    }
    result
}

pub fn relex<L: Lexer>(
    lexer: &L,
    old_tokens: &[Token],
    edit: &TextEdit,
    new_text: Text,
) -> (Vec<Token>, usize) {
    if old_tokens.iter().any(|&token| token.ty == ERROR) {
        return (lex(lexer, new_text), 0);
    }

    let mut old_tokens = old_tokens.iter().cloned();
    let mut old_len = tu(0);

    let mut new_tokens: Vec<Token> = Vec::new();
    let mut new_len = tu(0);

    let mut edit_point = tu(0);
    let mut reused = tu(0);

    for op in edit.ops.iter() {
        match *op {
            TextEditOp::Insert(ref buf) => edit_point += buf.as_text().len(),
            TextEditOp::Copy(range) => {
                let mut txt = new_text.slice(TextSuffix::from(new_len));
                while new_len < edit_point {
                    let token = lexer.step(&mut txt);
                    new_len += token.len;
                    new_tokens.push(token)
                }

                while old_len < range.start() {
                    old_len += old_tokens.next().unwrap().len;
                }

                loop {
                    let new_consumed = new_len - edit_point;
                    let old_consumed = old_len - range.start();
                    if new_consumed >= range.len() || old_consumed >= range.len() {
                        break;
                    }

                    match new_consumed.cmp(&old_consumed) {
                        Ordering::Less => {
                            let token = lexer.step(&mut txt);
                            new_len += token.len;
                            new_tokens.push(token)
                        }
                        Ordering::Equal => {
                            for token in &mut old_tokens {
                                old_len += token.len;
                                if old_len >= range.end() {
                                    break;
                                }
                                reused += token.len;
                                new_len += token.len;
                                new_tokens.push(token);
                            }
                        }
                        Ordering::Greater => {
                            let token = old_tokens.next().unwrap();
                            old_len += token.len;
                        }
                    }
                }

                edit_point += range.len()
            }
        }
    }

    let mut txt = new_text.slice(TextSuffix::from(new_len));
    while !txt.is_empty() {
        new_tokens.push(lexer.step(&mut txt));
    }
    let relexed_region = (new_text.len() - reused).utf8_len();
    (new_tokens, relexed_region)
}

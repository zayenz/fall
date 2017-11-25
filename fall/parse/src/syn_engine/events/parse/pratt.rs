use PrattTable;
use super::{Parser, TokenSeq, parse_any, parse_expr};

pub(super) fn parse_pratt<'g, 't>(
    p: &mut Parser<'g>,
    table: &'g PrattTable,
    tokens: TokenSeq<'t>,
) -> Option<TokenSeq<'t>> {
    go(p, table, tokens, 0)
}

fn go<'g, 't>(
    p: &mut Parser<'g>,
    table: &'g PrattTable,
    tokens: TokenSeq<'t>,
    min_prior: u32
) -> Option<TokenSeq<'t>> {
    let mut lhs = p.mark();
    let mut tokens = match prefix(p, table, tokens) {
        Some(ts) => ts,
        _ => return None,
    };

    'l: loop {
        for ix in table.infixes(min_prior) {
            let new_lhs = p.mark();
            let mark = p.start(ix.ty);
            if let Some(rest) = parse_expr(p, &ix.op, tokens) {
                tokens = rest;
                if ix.has_rhs {
                    if let Some(rest) = go(p, table, tokens, ix.priority + 1) {
                        tokens = rest;
                    } else {
                        p.start_error();
                        p.finish();
                    }
                }
                let ty = p.node_type(ix.ty);
                p.prev = Some(ty);
                p.forward_parent(lhs);
                lhs = new_lhs;
                p.finish();
                continue 'l;
            }
            p.rollback(mark)
        }
        break
    }

    Some(tokens)
}

fn prefix<'t, 'p>(
    p: &mut Parser<'p>,
    table: &'p PrattTable,
    tokens: TokenSeq<'t>,
) -> Option<TokenSeq<'t>> {
    if let Some(result) = parse_any(p, table.atoms.iter(), tokens) {
        return Some(result);
    }
    for prefix in table.prefixes.iter() {
        let mark = p.start(prefix.ty);
        if let Some(tokens) = parse_expr(p, &prefix.op, tokens) {
            if let Some(rest) = go(p, table, tokens, prefix.priority) {
                p.prev = Some(p.node_type(prefix.ty));
                p.finish();
                return Some(rest);
            }
        }
        p.rollback(mark);
    }
    None
}

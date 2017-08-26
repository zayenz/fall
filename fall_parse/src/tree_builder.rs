use elapsed::measure_time;

use fall_tree::{NodeType, ERROR, TextRange, FileStats, INode, TextUnit, Language};
use super::LexRule;
use lex_engine::{Token, tokenize};


#[derive(Clone, Copy, Debug)]
pub struct TokenSequence<'a> {
    text: &'a str,
    start: usize,
    non_ws_indexes: &'a [usize],
    original_tokens: &'a [Token],
}

impl<'a> TokenSequence<'a> {
    pub fn prefix(&self, suffix: TokenSequence<'a>) -> TokenSequence<'a> {
        TokenSequence {
            text: self.text,
            start: self.start,
            non_ws_indexes: &self.non_ws_indexes[..suffix.start - self.start],
            original_tokens: self.original_tokens
        }
    }

    pub fn current(&self) -> Option<Token> {
        self.non_ws_indexes.first().map(|&idx| {
            self.original_tokens[idx]
        })
    }

    pub fn bump(&self) -> (Node, TokenSequence<'a>) {
        let token = self.current().expect("Can't bump an empty token sequence");
        let node = Node::Leaf { ty: Some(token.ty), token_idx: self.non_ws_indexes[0] };
        let mut text_len = 0;
        let next_idx = self.non_ws_indexes.get(1).map(|&i| i).unwrap_or(self.original_tokens.len());
        for i in self.non_ws_indexes[0]..next_idx {
            text_len += self.original_tokens[i].len.as_u32() as usize;
        }
        let rest = TokenSequence {
            text: &self.text[text_len..],
            start: self.start + 1,
            non_ws_indexes: &self.non_ws_indexes[1..],
            original_tokens: self.original_tokens,
        };
        (node, rest)
    }

    pub fn bump_by_text(&self, ty: NodeType, text: &str) -> Option<(Node, TokenSequence<'a>)> {
        if self.text.starts_with(text) {
            let mut children = Vec::new();
            let mut leftover = TextUnit::from_usize(text.len());
            let mut rest = *self;
            while leftover > TextUnit::zero() {
                leftover -= rest.current().unwrap().len;
                let p = rest.bump();
                children.push(match p.0 {
                    Node::Leaf { ty: _, token_idx } => Node::Leaf { ty: None, token_idx: token_idx },
                    _ => unreachable!()
                });
                rest = p.1;
            }

            Some((Node::Composite { ty: Some(ty), children: children }, rest))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Leaf {
        ty: Option<NodeType>,
        token_idx: usize
    },
    Composite {
        ty: Option<NodeType>,
        children: Vec<Node>,
    }
}

impl Node {
    pub fn error() -> Node {
        Self::composite(Some(ERROR))
    }

    pub fn composite(ty: Option<NodeType>) -> Node {
        Node::Composite { ty: ty, children: Vec::new() }
    }

    pub fn success<'t>(ts: TokenSequence<'t>) -> (Node, TokenSequence<'t>) {
        (Self::composite(None), ts)
    }

    pub fn push_child(&mut self, child: Node) {
        match *self {
            Node::Composite { ref mut children, .. } => children.push(child),
            Node::Leaf { .. } => panic!("Can't add children to a leaf node"),
        }
    }

    pub fn debug(&self, tokens: &TokenSequence) -> String {
        let (l, r) = match (self.left_idx(), self.right_idx()) {
            (Some(l), Some(r)) => (l, r),
            _ => return "EMPTY-NODE".to_owned()
        };
        let mut result = String::new();
        let mut start = tokens.original_tokens[..l].iter().map(|t| t.len).sum::<TextUnit>();
        for t in tokens.original_tokens[l..r].iter() {
            result += &tokens.text[TextRange::from_len(start, t.len)];
            start += t.len;
        }
        result
    }

    fn right_idx(&self) -> Option<usize> {
        match *self {
            Node::Leaf { token_idx, .. } => Some(token_idx),
            Node::Composite { ref children, .. } =>
                children.iter().rev()
                    .filter_map(|n| n.right_idx())
                    .next(),
        }
    }

    fn left_idx(&self) -> Option<usize> {
        match *self {
            Node::Leaf { token_idx, .. } => Some(token_idx),
            Node::Composite { ref children, .. } =>
                children.first()
                    .and_then(|child| child.left_idx()),
        }
    }
}

fn is_ws(lang: &Language, token: Token) -> bool {
    lang.node_type_info(token.ty).whitespace_like
}

pub fn parse(
    text: &str,
    lang: &Language,
    tokenizer: &[LexRule],
    parser: &Fn(TokenSequence, &mut FileStats) -> Node
) -> (FileStats, INode) {
    let mut stats = FileStats::new();
    let (lex_time, owned_tokens) = measure_time(|| tokenize(&text, tokenizer).collect::<Vec<_>>());
    stats.lexing_time = lex_time.duration();
    stats.reparsed_region = TextRange::from_to(TextUnit::zero(), TextUnit::from_usize(text.len()));
    let non_ws_indexes: Vec<usize> = owned_tokens.iter().enumerate().filter_map(|(i, &t)| {
        if is_ws(lang, t) { None } else { Some(i) }
    }).collect();

    let ws_len = owned_tokens.iter()
        .take_while(|&&t| is_ws(lang, t))
        .map(|t| t.len.as_u32() as usize)
        .sum();
    let (parse_time, node) = {
        let tokens = TokenSequence {
            text: &text[ws_len..],
            start: 0,
            non_ws_indexes: &non_ws_indexes,
            original_tokens: &owned_tokens,
        };
        measure_time(|| parser(tokens, &mut stats))
    };
    stats.parsing_time = parse_time.duration();

    let ws_node = to_ws_node(lang, node, &owned_tokens);
    let inode = ws_node.into_inode().unwrap();
    (stats, inode)
}

#[derive(Debug)]
struct WsNode {
    ty: Option<NodeType>,
    len: TextUnit,
    children: Vec<WsNode>,
    first: Option<usize>,
    last: Option<usize>
}

impl WsNode {
    fn push_child(&mut self, lang: &Language, child: WsNode, tokens: &[Token]) {
        match (self.last, child.first) {
            (Some(l), Some(r)) if l + 1 < r => {
                for idx in l + 1..r {
                    let t = tokens[idx];
                    assert!(is_ws(lang, t), "expected whitespace, got {:?}", t.ty);
                    self.push_child_raw(token_ws_node(idx, t))
                }
            }
            _ => {}
        }
        self.push_child_raw(child)
    }

    fn push_child_raw(&mut self, child: WsNode) {
        self.last = child.last.or(self.last);
        self.first = self.first.or(child.first);
        self.len += child.len;
        self.children.push(child);
    }

    fn attach_to_inode(self, parent: &mut INode) {
        if self.children.is_empty() {
            match self.ty {
                Some(ty) => parent.push_child(INode::new_leaf(ty, self.len)),
                None => parent.push_token_part(self.len),
            }
            return;
        }
        match self.into_inode() {
            Ok(node) => parent.push_child(node),
            Err(this) => {
                for child in this.children {
                    child.attach_to_inode(parent);
                }
            }
        }
    }

    fn into_inode(self) -> Result<INode, WsNode> {
        if let Some(ty) = self.ty {
            let mut node = INode::new(ty);
            for child in self.children {
                child.attach_to_inode(&mut node);
            }
            Ok(node)
        } else {
            Err(self)
        }
    }
}

fn token_ws_node(idx: usize, t: Token) -> WsNode {
    WsNode {
        ty: Some(t.ty),
        len: t.len,
        children: Vec::new(),
        first: Some(idx),
        last: Some(idx),
    }
}

fn to_ws_node(lang: &Language, file_node: Node, tokens: &[Token]) -> WsNode {
    let (ty, children) = match file_node {
        Node::Composite { ty, children, .. } => (ty.unwrap(), children),
        _ => panic!("Root node must be composite")
    };
    let mut result = WsNode {
        ty: Some(ty),
        len: TextUnit::zero(),
        children: Vec::new(),
        first: None,
        last: None,
    };

    for (i, &t) in tokens.iter().enumerate() {
        if !is_ws(lang, t) {
            break
        }
        result.push_child_raw(token_ws_node(i, t))
    }

    for child in children {
        add_child(lang, &mut result, &child, tokens)
    }
    if let Some(idx) = result.last {
        for idx in idx + 1..tokens.len() {
            let t = tokens[idx];
            assert!(is_ws(lang, t));
            result.push_child_raw(token_ws_node(idx, t))
        }
    }
    result
}

fn add_child(lang: &Language, parent: &mut WsNode, node: &Node, tokens: &[Token]) {
    match *node {
        Node::Leaf { ty, token_idx } => {
            let mut node = token_ws_node(token_idx, tokens[token_idx]);
            node.ty = ty;
            parent.push_child(lang, node, tokens)
        }
        Node::Composite { ty, ref children } => {
            let mut p = WsNode {
                ty: ty,
                len: TextUnit::zero(),
                children: Vec::new(),
                first: None,
                last: None,
            };
            for child in children {
                add_child(lang, &mut p, child, tokens);
            }
            parent.push_child(lang, p, tokens)
        }
    }
}

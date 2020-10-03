use crate::EditorFileImpl;
use fall_tree::search::{find_leaf_at_offset, sibling, Direction, LeafAtOffset};
use fall_tree::test_util;
use fall_tree::{File, FileEdit, Node, TextEdit, TextRange, TextUnit};

pub fn default_context_actions(file: &File, range: TextRange, actions: &mut Vec<&'static str>) {
    for &(action_id, action) in DEFAULT_ACTIONS.iter() {
        if action(file, range.start(), false).is_some() {
            actions.push(action_id)
        }
    }
}

pub fn apply_default_context_action(file: &File, range: TextRange, id: &str) -> Option<TextEdit> {
    let action = DEFAULT_ACTIONS.iter().find(|&&(aid, _)| aid == id)?.1;
    action(file, range.start(), true).map(ActionResult::into_edit)
}
pub type ActionUnitItem<'a> = (&'a str, fn(&File, TextUnit, bool) -> Option<ActionResult>);
pub type ActionRangeItem<'a> = (&'a str, fn(&File, TextRange, bool) -> Option<ActionResult>);
pub const DEFAULT_ACTIONS: &[ActionUnitItem] = &[("Swap", swap)];

pub enum ActionResult {
    Available,
    Applied(TextEdit),
}

impl ActionResult {
    pub fn into_edit(self) -> TextEdit {
        match self {
            ActionResult::Available => {
                panic!("Context action should provide edit when apply is set to true")
            }
            ActionResult::Applied(edit) => edit,
        }
    }
}

pub fn check_context_action<E: EditorFileImpl>(action_id: &str, before: &str, after: &str) {
    let (before, range) = test_util::extract_range(before, "^");
    let file = E::parse(&before);
    let actions = file.context_actions(range);
    if !actions.contains(&action_id) {
        panic!("Action `{}` is not avialable", action_id);
    }
    match file.apply_context_action(range, action_id) {
        None => panic!("Failed to apply `{}` action", action_id),
        Some(edit) => {
            let actual = edit.apply(file.file().text());
            test_util::report_diff(after.trim(), actual.as_text().to_cow().trim())
        }
    }
}

pub fn check_no_context_action<E: EditorFileImpl>(action_id: &str, text: &str) {
    let (before, range) = test_util::extract_range(text, "^");
    let file = E::parse(&before);
    let actions = file.context_actions(range);
    if actions.contains(&action_id) {
        panic!("Action `{}` is avialable", action_id);
    }
}

fn swap(file: &File, offset: TextUnit, apply: bool) -> Option<ActionResult> {
    let comma = find_comma(file.root(), offset)?;
    let left = nonws_sibling(comma, Direction::Left)?;
    let right = nonws_sibling(comma, Direction::Right)?;
    if left.ty() != right.ty() {
        return None;
    }

    if !apply {
        return Some(ActionResult::Available);
    }
    let mut edit = FileEdit::new(file);
    edit.replace(left, right);
    edit.replace(right, left);
    Some(ActionResult::Applied(edit.into_text_edit()))
}

fn nonws_sibling(node: Node, direction: Direction) -> Option<Node> {
    let lang = node.file().language();
    let mut node = sibling(node, direction)?;
    while lang.node_type_info(node.ty()).whitespace_like {
        node = sibling(node, direction)?;
    }
    Some(node)
}

fn find_comma(node: Node, offset: TextUnit) -> Option<Node> {
    fn is_comma(node: Node) -> bool {
        node.text() == ","
    }
    match find_leaf_at_offset(node, offset) {
        LeafAtOffset::None => None,
        LeafAtOffset::Single(node) => {
            if is_comma(node) {
                Some(node)
            } else {
                None
            }
        }
        LeafAtOffset::Between(left, right) => match (is_comma(left), is_comma(right)) {
            (true, _) => Some(left),
            (_, true) => Some(right),
            _ => None,
        },
    }
}

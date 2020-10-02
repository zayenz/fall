use fall_editor::actions::ActionResult;
use fall_tree::{File, TextRange};

mod extract_rule;
mod swap_alternatives;

pub const ACTIONS: &[(&str, fn(&File, TextRange, bool) -> Option<ActionResult>)] = &[
    ("Swap alternatives", |file, range, apply| {
        swap_alternatives::swap_alternatives(file, range.start(), apply)
    }),
    ("Extract rule", extract_rule::extract_rule),
];

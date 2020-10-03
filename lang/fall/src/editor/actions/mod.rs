use fall_editor::actions::ActionRangeItem;

mod extract_rule;
mod swap_alternatives;

pub const ACTIONS: &[ActionRangeItem] = &[
    ("Swap alternatives", |file, range, apply| {
        swap_alternatives::swap_alternatives(file, range.start(), apply)
    }),
    ("Extract rule", extract_rule::extract_rule),
];

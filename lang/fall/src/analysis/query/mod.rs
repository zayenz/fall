use std::collections::HashMap;
use std::sync::Arc;

use fall_tree::Text;

use super::db::Query;
use crate::syntax::{
    AstClassDef, AstNodeDef, AstTraitDef, CallExpr, Expr, LexRule, MethodDef, Parameter, RefExpr,
    SynRule,
};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct AllLexRules;
mod all_lex_rules;
impl<'f> Query<'f> for AllLexRules {
    type Result = Arc<HashMap<Text<'f>, LexRule<'f>>>;
}

#[derive(Debug)]
pub(crate) struct FindLexRule<'f>(pub Text<'f>);
mod find_lex_rule;
impl<'f> Query<'f> for FindLexRule<'f> {
    type Result = Option<LexRule<'f>>;
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct AllSynRules;
mod all_syn_rules;
impl<'f> Query<'f> for AllSynRules {
    type Result = Arc<HashMap<Text<'f>, SynRule<'f>>>;
}

#[derive(Debug)]
pub(crate) struct FindSynRule<'f>(pub Text<'f>);
mod find_syn_rule;
impl<'f> Query<'f> for FindSynRule<'f> {
    type Result = Option<SynRule<'f>>;
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct UnusedRules;
mod unused_rules;
impl<'f> Query<'f> for UnusedRules {
    type Result = ();
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct AllContexts;
mod all_context_ids;
impl<'f> Query<'f> for AllContexts {
    type Result = Arc<Vec<Text<'f>>>;
}

#[derive(Copy, Clone)]
pub enum RefKind<'f> {
    Token(LexRule<'f>),
    RuleReference(SynRule<'f>),
    Param(Parameter<'f>),
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct ResolveRefExpr<'f>(pub RefExpr<'f>);
mod resolve_ref_expr;
impl<'f> Query<'f> for ResolveRefExpr<'f> {
    type Result = Option<RefKind<'f>>;
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CallKind<'f> {
    Eof,
    Any,
    Commit,

    Not(Expr<'f>),
    Layer(Expr<'f>, Expr<'f>),
    WithSkip(Expr<'f>, Expr<'f>),

    Enter(u32, Expr<'f>),
    Exit(u32, Expr<'f>),
    IsIn(u32),

    RuleCall(SynRule<'f>, Arc<Vec<(Parameter<'f>, Expr<'f>)>>),
    PrevIs(Arc<Vec<SynRule<'f>>>),
    Inject(Expr<'f>, Expr<'f>),
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct ResolveCall<'f>(pub CallExpr<'f>);
mod resolve_call;
impl<'f> Query<'f> for ResolveCall<'f> {
    type Result = Option<CallKind<'f>>;
}

#[derive(Copy, Clone)]
pub enum PratVariant<'f> {
    Atom(Expr<'f>),
    Bin(PrattOp<'f>),
    Postfix(PrattOp<'f>),
    Prefix(PrattOp<'f>),
}

#[derive(Copy, Clone)]
pub struct PrattOp<'f> {
    pub op: Expr<'f>,
    pub priority: u32,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct ResolvePrattVariant<'f>(pub SynRule<'f>);
mod resolve_pratt_variant;
impl<'f> Query<'f> for ResolvePrattVariant<'f> {
    type Result = Option<PratVariant<'f>>;
}

#[derive(Copy, Clone)]
pub enum Arity {
    Single,
    Optional,
    Many,
}

#[derive(Copy, Clone)]
pub enum ChildKind<'f> {
    AstNode(AstNodeDef<'f>),
    AstClass(AstClassDef<'f>),
    Token(LexRule<'f>),
}

#[derive(Copy, Clone)]
pub enum MethodKind<'f> {
    NodeAccessor(ChildKind<'f>, Arity),
    TextAccessor(LexRule<'f>, Arity),
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct ResolveMethod<'f>(pub MethodDef<'f>);
mod resolve_method;
impl<'f> Query<'f> for ResolveMethod<'f> {
    type Result = Option<MethodKind<'f>>;
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub(crate) struct AstNodeTraits<'f>(pub AstNodeDef<'f>);
mod ast_node_traits;
impl<'f> Query<'f> for AstNodeTraits<'f> {
    type Result = Arc<Vec<AstTraitDef<'f>>>;
}

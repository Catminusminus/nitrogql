use super::{
    base::{Ident, Pos, Variable},
    directive::Directive,
    r#type::Type,
    selection_set::SelectionSet,
    value::Value,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Clone, Debug)]
pub struct VariablesDefinition<'a> {
    pub position: Pos,
    pub definitions: Vec<VariableDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct VariableDefinition<'a> {
    pub pos: Pos,
    pub name: Variable<'a>,
    pub r#type: Type<'a>,
    pub default_value: Option<Value<'a>>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Clone, Debug)]
pub enum ExecutableDefinition<'a> {
    OperationDefinition(OperationDefinition<'a>),
    FragmentDefinition(FragmentDefinition<'a>),
}

#[derive(Clone, Debug)]
pub struct OperationDefinition<'a> {
    pub operation_type: OperationType,
    pub name: Option<Ident<'a>>,
    pub variables_definition: Option<VariablesDefinition<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

#[derive(Clone, Debug)]
pub struct FragmentDefinition<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub type_condition: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

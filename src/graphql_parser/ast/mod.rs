use self::{
    base::Ident,
    directive::Directive,
    operations::{ExecutableDefinition, OperationType, VariablesDefinition},
    selection_set::SelectionSet,
};

pub mod base;
pub mod directive;
pub mod operations;
pub mod selection_set;
pub mod r#type;
pub mod value;

#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    pub definitions: Vec<ExecutableDefinition<'a>>,
}

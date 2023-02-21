use super::{base::build_variable, utils::PairExt, Rule};
use pest::iterators::Pair;

use crate::{
    graphql_parser::ast::{
        base::Pos,
        value::{
            BooleanValue, EnumValue, FloatValue, IntValue, ListValue, NullValue, ObjectValue,
            StringValue, Value,
        },
    },
    parts,
};

/// Builds Value from given Pair for Value.
pub fn build_value(pair: Pair<Rule>) -> Value {
    let pair = pair.only_child();
    let position: Pos = (&pair).into();
    match pair.as_rule() {
        Rule::Variable => Value::Variable(build_variable(pair.only_child())),
        Rule::IntValue => Value::IntValue(IntValue {
            position,
            value: pair.as_str(),
        }),
        Rule::FloatValue => Value::FloatValue(FloatValue {
            position,
            value: pair.as_str(),
        }),
        Rule::StringValue => Value::StringValue(StringValue {
            position,
            value: pair.as_str(),
        }),
        Rule::BooleanValue => Value::BooleanValue(BooleanValue {
            position,
            keyword: pair.as_str(),
        }),
        Rule::NullValue => Value::NullValue(NullValue {
            position,
            keyword: pair.as_str(),
        }),
        Rule::EnumValue => Value::EnumValue(EnumValue {
            position,
            value: pair.as_str(),
        }),
        Rule::ListValue => {
            let values = pair
                .all_children(Rule::Value)
                .into_iter()
                .map(build_value)
                .collect();
            Value::ListValue(ListValue { position, values })
        }
        Rule::ObjectValue => {
            let fields = pair
                .all_children(Rule::ObjectField)
                .into_iter()
                .map(|field| {
                    let (name, value) = parts!(field.into_inner(), Name, Value);
                    (name.into(), build_value(value))
                })
                .collect();
            Value::ObjectValue(ObjectValue { position, fields })
        }
        rule => panic!("Unexpected rule {:?} as a child of Value", rule),
    }
}

use graphql_parser::query::{
    Definition, Document, OperationDefinition, Query, Selection, SelectionSet, TypeCondition,
};

use crate::{source_map_writer::writer::SourceMapWriter, utils::capitalize::capitalize};

use super::{printer::QueryTypePrinterOptions, ts_types::TSType, ts_types_util::ts_intersection};

pub trait TypePrinter {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter);
}

impl TypePrinter for Document<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        for d in self.definitions.iter() {
            d.print_type(options, writer);
        }
    }
}

impl TypePrinter for Definition<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        match self {
            Definition::Operation(ref op) => match op {
                OperationDefinition::Query(ref query) => query.print_type(options, writer),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl TypePrinter for Query<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        let query_name = self
            .name
            .as_ref()
            .map(|name| capitalize(name))
            .unwrap_or(String::new());
        let query_type_name = format!("{}{}", query_name, options.query_result_suffix);

        writer.write("type ");
        writer.write(&query_type_name);
        writer.write(" = ");
        get_type_for_selection_set(
            &self.selection_set,
            TSType::TypeVariable(options.schema_root.clone()),
        )
        .print_type(options, writer);
        writer.write(";\n\n");
    }
}

fn get_type_for_selection_set(
    selection_set: &SelectionSet<'_, String>,
    parent_type: TSType,
) -> TSType {
    if selection_set.items.is_empty() {
        // empty selection set means selecting the whole parent
        return parent_type;
    }
    let types_for_each_field = selection_set
        .items
        .iter()
        .map(|sel| match sel {
            Selection::Field(ref field) => {
                let property_name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                let key = TSType::StringLiteral(field.name.clone());
                let field_type = TSType::IndexType(Box::new(parent_type.clone()), Box::new(key));
                let field_sel_type = get_type_for_selection_set(&field.selection_set, field_type);
                TSType::Object(vec![(property_name, field_sel_type)])
            }
            Selection::FragmentSpread(ref fragment) => {
                TSType::TypeVariable(fragment.fragment_name.clone())
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => get_type_for_selection_set(&fragment.selection_set, parent_type.clone()),
                Some(TypeCondition::On(ref cond)) =>
                // TODO: this isn't correct
                {
                    get_type_for_selection_set(
                        &fragment.selection_set,
                        TSType::TypeVariable(cond.clone()),
                    )
                }
            },
        })
        .collect();
    ts_intersection(types_for_each_field)
}

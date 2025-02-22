use std::{borrow::Borrow, fmt::Display};

use crate::{
    ts_types::{
        ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type, ObjectField, TSType,
    },
    utils::interface_implementers,
};
use nitrogql_ast::{
    base::{Ident, Pos},
    type_system::{
        EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
        ObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition, TypeSystemDefinition,
        TypeSystemDocument, UnionTypeDefinition,
    },
    value::StringValue,
};
use sourcemap_writer::SourceMapWriter;

use crate::jsdoc::print_description as jsdoc_print_description;

use super::{
    error::{SchemaTypePrinterError, SchemaTypePrinterResult},
    printer::SchemaTypePrinterContext,
};

pub trait TypePrinter {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()>;
}

impl TypePrinter for TypeSystemDocument<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let schema_metadata_type = get_schema_metadata_type(self);
        writer.write("export type ");
        writer.write(&context.options.schema_metadata_type);
        writer.write(" = ");
        schema_metadata_type.print_type(writer);
        writer.write(";\n\n");
        // Print utility types
        writer.write(
            "type __Beautify<Obj> = { [K in keyof Obj]: Obj[K] } & {};
export type __SelectionSet<Orig, Obj, Others> =
  __Beautify<Pick<{
    [K in keyof Orig]: Obj extends Record<K, infer V> ? V : unknown
  }, Extract<keyof Orig, keyof Obj>> & Others>;
",
        );

        for def in self.definitions.iter() {
            def.print_type(context, writer)?;
            writer.write("\n");
        }
        Ok(())
    }
}

fn get_schema_metadata_type(document: &TypeSystemDocument) -> TSType {
    let schema_definition = document.definitions.iter().find_map(|def| match def {
        TypeSystemDefinition::SchemaDefinition(def) => Some(def),
        _ => None,
    });
    if let Some(schema_def) = schema_definition {
        return TSType::object(schema_def.definitions.iter().map(|(op, ty)| {
            (
                op.as_str(),
                TSType::TypeVariable(ty.into()),
                schema_def.description.as_ref().map(|d| d.value.clone()),
            )
        }));
    }
    // If there is no schema definition, use default root type names.
    let mut operations = vec![];
    for d in document.definitions.iter() {
        let TypeSystemDefinition::TypeDefinition(ref def) = d else {
            continue;
        };
        let TypeDefinition::Object(ref def) = def else{
            continue;
        };

        match def.name.name {
            "Query" => {
                operations.push(("query", (&def.name).into()));
            }
            "Mutation" => {
                operations.push(("mutation", (&def.name).into()));
            }
            "Subscription" => {
                operations.push(("subscription", (&def.name).into()));
            }
            _ => {}
        }
    }

    TSType::object(
        operations
            .into_iter()
            .map(|(op, ty)| (op, TSType::TypeVariable(ty), None)),
    )
}

impl TypePrinter for TypeSystemDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeSystemDefinition::SchemaDefinition(_) => Ok(()),
            TypeSystemDefinition::TypeDefinition(def) => def.print_type(context, writer),
            TypeSystemDefinition::DirectiveDefinition(_) => Ok(()),
        }
    }
}

impl TypePrinter for TypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeDefinition::Scalar(def) => def.print_type(context, writer),
            TypeDefinition::Object(def) => def.print_type(context, writer),
            TypeDefinition::Interface(def) => def.print_type(context, writer),
            TypeDefinition::Union(def) => def.print_type(context, writer),
            TypeDefinition::Enum(def) => def.print_type(context, writer),
            TypeDefinition::InputObject(def) => def.print_type(context, writer),
        }
    }
}

impl TypePrinter for ScalarTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let Some(scalar_type_str) = context.options.scalar_types.get(self.name.name) else {
            return Err(SchemaTypePrinterError::ScalarTypeNotProvided {
                position: self.position,
                name: self.name.to_string(),
            });
        };

        print_description(&self.description, writer);
        // Special casing for reexport
        if self.name.name == scalar_type_str {
            let tmp_type_name = format!("__tmp_{}", self.name);
            writer.write_for("type ", &self.scalar_keyword);
            writer.write_for(&tmp_type_name, &self.name);
            writer.write(" = ");
            writer.write(scalar_type_str);
            writer.write(";\nexport type { ");
            writer.write(&tmp_type_name);
            writer.write(" as ");
            writer.write(self.name.name);
            writer.write("};\n");
        } else {
            writer.write_for("export type ", &self.scalar_keyword);
            writer.write_for(self.name.name, &self.name);
            writer.write(" = ");
            writer.write(scalar_type_str);
            writer.write(";\n");
        }
        Ok(())
    }
}

impl TypePrinter for ObjectTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let type_name_ident = Ident {
            name: "__typename",
            position: Pos::builtin(),
        };
        let schema_type = context.schema.get_type(self.name.name);
        let obj_type = TSType::object(
            vec![(
                &type_name_ident,
                TSType::StringLiteral(self.name.to_string()),
                None,
            )]
            .into_iter()
            .chain(self.fields.iter().map(|field| {
                let schema_field = schema_type
                    .and_then(|ty| ty.as_object())
                    .and_then(|ty| ty.fields.iter().find(|f| f.name == field.name.name));
                (
                    &field.name,
                    get_ts_type_of_type(&field.r#type, |name| {
                        TSType::TypeVariable((&name.name).into())
                    }),
                    make_ts_description(
                        &field.description,
                        &schema_field.and_then(|f| f.deprecation.as_ref()),
                    ),
                )
            })),
        );

        print_description(&self.description, writer);
        writer.write_for("export type ", &self.type_keyword);
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        obj_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for InterfaceTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        // In generated type definitions, an interface is expressed as a union of all possible concrete types.
        let union_constituents =
            interface_implementers(context.schema, self.name.name).map(|obj| {
                TSType::TypeVariable({
                    let s: &str = (obj.name).inner_ref().borrow();
                    s.into()
                })
            });
        let intf_type = ts_union(union_constituents);

        print_description(&self.description, writer);
        writer.write_for("export type ", &self.interface_keyword);
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        intf_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for UnionTypeDefinition<'_> {
    fn print_type(
        &self,
        _context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let union_type = ts_union(
            self.members
                .iter()
                .map(|mem| TSType::TypeVariable(mem.into())),
        );

        print_description(&self.description, writer);
        writer.write_for("export type ", &self.union_keyword);
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        union_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for EnumTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let enum_type = TSType::Union(
            self.values
                .iter()
                .map(|mem| TSType::StringLiteral(mem.name.to_string()))
                .collect(),
        );

        print_description(&self.description, writer);
        writer.write_for("export type ", &self.enum_keyword);
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        enum_type.print_type(writer);
        writer.write(";\n");

        if context.options.emit_schema_runtime {
            writer.write_for("export const ", &self.enum_keyword);
            writer.write_for(self.name.name, &self.name);
            writer.write(" = {\n");
            for value in &self.values {
                writer.write_for(value.name.name, &value.name);
                writer.write(": \"");
                writer.write_for(value.name.name, &value.name);
                writer.write("\",\n");
            }
            writer.write("} as const;\n")
        }
        Ok(())
    }
}

impl TypePrinter for InputObjectTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let schema_type = context.schema.get_type(self.name.name);
        let obj_type = TSType::Object(
            self.fields
                .iter()
                .map(|field| {
                    let schema_field = schema_type
                        .and_then(|t| t.as_input_object())
                        .and_then(|t| t.fields.iter().find(|f| f.name == field.name.name))
                        .expect("Type system error");

                    let ts_type = get_ts_type_of_type(&field.r#type, |name| {
                        TSType::TypeVariable((&name.name).into())
                    })
                    .into_readonly();
                    ObjectField {
                        key: (&field.name).into(),
                        r#type: ts_type,
                        readonly: true,
                        optional: context.options.input_nullable_field_is_optional
                            && !field.r#type.is_nonnull(),
                        description: make_ts_description(
                            &field.description,
                            &schema_field.deprecation,
                        ),
                    }
                })
                .collect(),
        );

        print_description(&self.description, writer);
        writer.write_for("export type ", &self.input_keyword);
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        obj_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

fn print_description(description: &Option<StringValue>, writer: &mut impl SourceMapWriter) {
    if let Some(description) = description {
        jsdoc_print_description(description, writer);
    }
}

/// Combines description and deprecation reason into a single string.
fn make_ts_description(
    description: &Option<StringValue>,
    deprecation: &Option<impl Display>,
) -> Option<String> {
    match (description, deprecation) {
        (Some(description), Some(deprecation)) => Some(format!(
            "{}\n\n@deprecated {}",
            description.value, deprecation
        )),
        (Some(description), None) => Some(description.value.clone()),
        (None, Some(deprecation)) => format!("@deprecated {}", deprecation).into(),
        (None, None) => None,
    }
}

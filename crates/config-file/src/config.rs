use std::{collections::HashMap, path::PathBuf, str::FromStr};

use serde::Deserialize;

use crate::parsing_utils::{default_true, deserialize_fromstr};

#[derive(Debug, Default)]
pub struct Config {
    /// Path(s) to schema definition files.
    pub schema: Vec<String>,
    /// Path(s) to operation definition files.
    pub operations: Vec<String>,
    // extensions
    pub generate: GenerateConfig,
}

/// Config related to the 'generate' command.
#[derive(Debug, Deserialize)]
pub struct GenerateConfig {
    /// Mode of generation.
    #[serde(deserialize_with = "deserialize_fromstr", default)]
    pub mode: GenerateMode,
    /// Output file path for schema.
    #[serde(rename = "schemaOutput")]
    pub schema_output: Option<PathBuf>,
    /// Module specifier for import schema types from operations.
    /// Defaults to relative paths.
    #[serde(rename = "schemaModuleSpecifier")]
    pub schema_module_specifier: Option<String>,
    /// Mapping from GraphQL scalar types to TypeScript types.
    #[serde(rename = "scalarTypes", default)]
    pub scalar_types: HashMap<String, String>,
    /// Whether operation is exported as a default export.
    /// Effective only when a document contains only one operation.
    #[serde(rename = "defaultExportForOperation", default = "default_true")]
    pub default_export_for_operation: bool,
    /// Config related to generated names.
    #[serde(default)]
    pub name: GenerateNameConfig,
    /// Whether to emit runtime for generated schema types.
    #[serde(rename = "emitSchemaRuntime", default)]
    pub emit_schema_runtime: bool,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        GenerateConfig {
            mode: Default::default(),
            schema_output: None,
            schema_module_specifier: None,
            scalar_types: Default::default(),
            default_export_for_operation: true,
            name: Default::default(),
            emit_schema_runtime: false,
        }
    }
}

/// Mode of code generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum GenerateMode {
    /// To be used with a webpack loader for .graphql files, emits .d.graphql.ts files that are supported by TS 5.0 and later
    #[default]
    WithLoaderTS5_0,
    /// To be used with a webpack loader for .graphql files, emits .d.graphql.ts files that are supported by TS 4.0
    WithLoaderTS4_0,
    /// To be used standalone. Emits .graphql.ts that are supported by TS 4.0
    StandaloneTS4_0,
}

pub struct FromStrError;

impl FromStr for GenerateMode {
    type Err = FromStrError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "with-loader-ts-5.0" => Ok(GenerateMode::WithLoaderTS5_0),
            "with-loader-ts-4.0" => Ok(GenerateMode::WithLoaderTS4_0),
            "standalone-ts-4.0" => Ok(GenerateMode::StandaloneTS4_0),
            _ => Err(FromStrError),
        }
    }
}

/// Config related to names of generated variables and types.
#[derive(Debug, Default, Deserialize)]
pub struct GenerateNameConfig {
    /// Suffix for type of operation result.
    #[serde(rename = "operationResultTypeSuffix")]
    pub operation_result_type_suffix: Option<String>,
    /// Suffix for type of variables for an operation.
    #[serde(rename = "variablesTypeSuffix")]
    pub variables_type_suffix: Option<String>,
    /// Whether operation name should be capitalized.
    #[serde(rename = "capitalizeOperationNames")]
    pub capitalize_operation_names: Option<bool>,
    /// Suffix for variable of query.
    #[serde(rename = "queryVariableSuffix")]
    pub query_variable_suffix: Option<String>,
    /// Suffix for variable of mutation.
    #[serde(rename = "mutationVariableSuffix")]
    pub mutation_variable_suffix: Option<String>,
    /// Suffix for variable of subscription.
    #[serde(rename = "subscriptionVariableSuffix")]
    pub subscription_variable_suffix: Option<String>,
}

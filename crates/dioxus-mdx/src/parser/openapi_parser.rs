//! OpenAPI specification parser.
//!
//! Parses OpenAPI 3.0/3.1 YAML or JSON specs into internal types for rendering.

use std::collections::BTreeMap;

use openapiv3::{
    OpenAPI, Operation, Parameter, ParameterSchemaOrContent, PathItem, ReferenceOr, RequestBody,
    Response, Schema, SchemaKind, StatusCode, Type, VariantOrUnknownOrEmpty,
};

use super::openapi_types::*;

/// Error type for OpenAPI parsing.
#[derive(Debug, Clone)]
pub enum OpenApiError {
    /// YAML/JSON parsing failed.
    ParseError(String),
    /// Invalid or unsupported spec structure.
    InvalidSpec(String),
}

impl std::fmt::Display for OpenApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::InvalidSpec(msg) => write!(f, "Invalid spec: {}", msg),
        }
    }
}

impl std::error::Error for OpenApiError {}

/// Parse an OpenAPI specification from YAML or JSON content.
pub fn parse_openapi(content: &str) -> Result<OpenApiSpec, OpenApiError> {
    // Try YAML first, then JSON
    let spec: OpenAPI = if let Ok(s) = serde_yaml::from_str(content) {
        s
    } else if let Ok(s) = serde_json::from_str(content) {
        s
    } else {
        return Err(OpenApiError::ParseError("Failed to parse as YAML or JSON".to_string()));
    };

    Ok(transform_spec(&spec))
}

/// Transform an openapiv3 spec into our internal representation.
fn transform_spec(spec: &OpenAPI) -> OpenApiSpec {
    let info = ApiInfo {
        title: spec.info.title.clone(),
        version: spec.info.version.clone(),
        description: spec.info.description.clone(),
    };

    let servers = spec
        .servers
        .iter()
        .map(|s| ApiServer {
            url: s.url.clone(),
            description: s.description.clone(),
        })
        .collect();

    let tags: Vec<ApiTag> = spec
        .tags
        .iter()
        .map(|t| ApiTag {
            name: t.name.clone(),
            description: t.description.clone(),
        })
        .collect();

    // Collect all operations from paths
    let mut operations = Vec::new();
    for (path, item) in &spec.paths.paths {
        if let ReferenceOr::Item(path_item) = item {
            extract_operations(path, path_item, spec, &mut operations);
        }
    }

    // Extract schemas
    let mut schemas = BTreeMap::new();
    if let Some(components) = &spec.components {
        for (name, schema_ref) in &components.schemas {
            if let ReferenceOr::Item(schema) = schema_ref {
                schemas.insert(name.clone(), transform_schema(schema, spec));
            }
        }
    }

    OpenApiSpec {
        info,
        servers,
        operations,
        tags,
        schemas,
    }
}

/// Extract operations from a path item.
fn extract_operations(
    path: &str,
    item: &PathItem,
    spec: &OpenAPI,
    operations: &mut Vec<ApiOperation>,
) {
    let methods = [
        (HttpMethod::Get, &item.get),
        (HttpMethod::Post, &item.post),
        (HttpMethod::Put, &item.put),
        (HttpMethod::Delete, &item.delete),
        (HttpMethod::Patch, &item.patch),
        (HttpMethod::Head, &item.head),
        (HttpMethod::Options, &item.options),
    ];

    for (method, op_option) in methods {
        if let Some(op) = op_option {
            operations.push(transform_operation(path, method, op, &item.parameters, spec));
        }
    }
}

/// Transform an operation.
fn transform_operation(
    path: &str,
    method: HttpMethod,
    op: &Operation,
    path_params: &[ReferenceOr<Parameter>],
    spec: &OpenAPI,
) -> ApiOperation {
    // Combine path-level and operation-level parameters
    let mut parameters: Vec<ApiParameter> = path_params
        .iter()
        .filter_map(|p| transform_parameter(p, spec))
        .collect();

    for param in &op.parameters {
        if let Some(p) = transform_parameter(param, spec) {
            // Don't add duplicates (operation params override path params)
            if !parameters.iter().any(|existing| existing.name == p.name) {
                parameters.push(p);
            }
        }
    }

    let request_body = op
        .request_body
        .as_ref()
        .and_then(|rb| transform_request_body(rb, spec));

    let responses = op
        .responses
        .responses
        .iter()
        .map(|(code, resp)| transform_response(code, resp, spec))
        .collect();

    ApiOperation {
        operation_id: op.operation_id.clone(),
        method,
        path: path.to_string(),
        summary: op.summary.clone(),
        description: op.description.clone(),
        tags: op.tags.clone(),
        parameters,
        request_body,
        responses,
        deprecated: op.deprecated,
    }
}

/// Transform a parameter.
fn transform_parameter(
    param_ref: &ReferenceOr<Parameter>,
    spec: &OpenAPI,
) -> Option<ApiParameter> {
    let param = resolve_parameter(param_ref, spec)?;

    let location = match &param.parameter_data_ref().format {
        openapiv3::ParameterSchemaOrContent::Schema(_) => {
            // Get location from the parameter kind
            match param {
                Parameter::Query { .. } => ParameterLocation::Query,
                Parameter::Header { .. } => ParameterLocation::Header,
                Parameter::Path { .. } => ParameterLocation::Path,
                Parameter::Cookie { .. } => ParameterLocation::Cookie,
            }
        }
        _ => return None,
    };

    let data = param.parameter_data_ref();
    let schema = match &data.format {
        ParameterSchemaOrContent::Schema(s) => Some(resolve_and_transform_schema(s, spec)),
        _ => None,
    };

    Some(ApiParameter {
        name: data.name.clone(),
        location,
        description: data.description.clone(),
        required: data.required,
        deprecated: data.deprecated.unwrap_or(false),
        schema,
        example: data.example.as_ref().map(|v| format_json_value(v)),
    })
}

/// Resolve a parameter reference.
fn resolve_parameter<'a>(
    param_ref: &'a ReferenceOr<Parameter>,
    spec: &'a OpenAPI,
) -> Option<&'a Parameter> {
    match param_ref {
        ReferenceOr::Item(param) => Some(param),
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/parameters/")?;
            spec.components
                .as_ref()?
                .parameters
                .get(name)
                .and_then(|p| match p {
                    ReferenceOr::Item(param) => Some(param),
                    _ => None,
                })
        }
    }
}

/// Transform a request body.
fn transform_request_body(
    rb_ref: &ReferenceOr<RequestBody>,
    spec: &OpenAPI,
) -> Option<ApiRequestBody> {
    let rb = resolve_request_body(rb_ref, spec)?;

    let content = rb
        .content
        .iter()
        .map(|(media_type, media)| MediaTypeContent {
            media_type: media_type.clone(),
            schema: media.schema.as_ref().map(|s| resolve_and_transform_schema(s, spec)),
            example: media.example.as_ref().map(|v| format_json_value(v)),
        })
        .collect();

    Some(ApiRequestBody {
        description: rb.description.clone(),
        required: rb.required,
        content,
    })
}

/// Resolve a request body reference.
fn resolve_request_body<'a>(
    rb_ref: &'a ReferenceOr<RequestBody>,
    spec: &'a OpenAPI,
) -> Option<&'a RequestBody> {
    match rb_ref {
        ReferenceOr::Item(rb) => Some(rb),
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/requestBodies/")?;
            spec.components
                .as_ref()?
                .request_bodies
                .get(name)
                .and_then(|r| match r {
                    ReferenceOr::Item(rb) => Some(rb),
                    _ => None,
                })
        }
    }
}

/// Transform a response.
fn transform_response(
    status_code: &StatusCode,
    resp_ref: &ReferenceOr<Response>,
    spec: &OpenAPI,
) -> ApiResponse {
    let status_str = match status_code {
        StatusCode::Code(code) => code.to_string(),
        StatusCode::Range(range) => format!("{}XX", range),
    };

    let resp = resolve_response(resp_ref, spec);

    let (description, content) = if let Some(r) = resp {
        let content = r
            .content
            .iter()
            .map(|(media_type, media)| MediaTypeContent {
                media_type: media_type.clone(),
                schema: media.schema.as_ref().map(|s| resolve_and_transform_schema(s, spec)),
                example: media.example.as_ref().map(|v| format_json_value(v)),
            })
            .collect();
        (r.description.clone(), content)
    } else {
        (String::new(), Vec::new())
    };

    ApiResponse {
        status_code: status_str,
        description,
        content,
    }
}

/// Resolve a response reference.
fn resolve_response<'a>(
    resp_ref: &'a ReferenceOr<Response>,
    spec: &'a OpenAPI,
) -> Option<&'a Response> {
    match resp_ref {
        ReferenceOr::Item(resp) => Some(resp),
        ReferenceOr::Reference { reference } => {
            let name = reference.strip_prefix("#/components/responses/")?;
            spec.components
                .as_ref()?
                .responses
                .get(name)
                .and_then(|r| match r {
                    ReferenceOr::Item(resp) => Some(resp),
                    _ => None,
                })
        }
    }
}

/// Resolve a schema reference and transform it.
fn resolve_and_transform_schema(schema_ref: &ReferenceOr<Schema>, spec: &OpenAPI) -> SchemaDefinition {
    match schema_ref {
        ReferenceOr::Item(schema) => transform_schema(schema, spec),
        ReferenceOr::Reference { reference } => {
            // Extract the reference name
            let ref_name = reference
                .strip_prefix("#/components/schemas/")
                .map(|s| s.to_string());

            // Try to resolve the schema
            let resolved = ref_name.as_ref().and_then(|name| {
                spec.components
                    .as_ref()?
                    .schemas
                    .get(name)
                    .and_then(|s| match s {
                        ReferenceOr::Item(schema) => Some(schema),
                        _ => None,
                    })
            });

            if let Some(schema) = resolved {
                let mut def = transform_schema(schema, spec);
                def.ref_name = ref_name;
                def
            } else {
                SchemaDefinition {
                    ref_name,
                    ..Default::default()
                }
            }
        }
    }
}

/// Resolve a boxed schema reference and transform it.
fn resolve_and_transform_boxed_schema(schema_ref: &ReferenceOr<Box<Schema>>, spec: &OpenAPI) -> SchemaDefinition {
    match schema_ref {
        ReferenceOr::Item(schema) => transform_schema(schema, spec),
        ReferenceOr::Reference { reference } => {
            // Extract the reference name
            let ref_name = reference
                .strip_prefix("#/components/schemas/")
                .map(|s| s.to_string());

            // Try to resolve the schema
            let resolved = ref_name.as_ref().and_then(|name| {
                spec.components
                    .as_ref()?
                    .schemas
                    .get(name)
                    .and_then(|s| match s {
                        ReferenceOr::Item(schema) => Some(schema),
                        _ => None,
                    })
            });

            if let Some(schema) = resolved {
                let mut def = transform_schema(schema, spec);
                def.ref_name = ref_name;
                def
            } else {
                SchemaDefinition {
                    ref_name,
                    ..Default::default()
                }
            }
        }
    }
}

/// Helper to extract format string from VariantOrUnknownOrEmpty.
fn extract_format<T: std::fmt::Debug>(format: &VariantOrUnknownOrEmpty<T>) -> Option<String> {
    match format {
        VariantOrUnknownOrEmpty::Item(f) => Some(format!("{:?}", f).to_lowercase()),
        VariantOrUnknownOrEmpty::Unknown(s) => Some(s.clone()),
        VariantOrUnknownOrEmpty::Empty => None,
    }
}

/// Transform a schema.
fn transform_schema(schema: &Schema, spec: &OpenAPI) -> SchemaDefinition {
    let mut def = SchemaDefinition::default();

    def.description = schema.schema_data.description.clone();
    def.example = schema.schema_data.example.as_ref().map(|v| format_json_value(v));
    def.default = schema.schema_data.default.as_ref().map(|v| format_json_value(v));
    def.nullable = schema.schema_data.nullable;

    match &schema.schema_kind {
        SchemaKind::Type(t) => {
            match t {
                Type::String(s) => {
                    def.schema_type = SchemaType::String;
                    def.format = extract_format(&s.format);
                    def.enum_values = s.enumeration.iter().filter_map(|v| v.clone()).collect();
                }
                Type::Number(n) => {
                    def.schema_type = SchemaType::Number;
                    def.format = extract_format(&n.format);
                }
                Type::Integer(i) => {
                    def.schema_type = SchemaType::Integer;
                    def.format = extract_format(&i.format);
                }
                Type::Boolean(_) => {
                    def.schema_type = SchemaType::Boolean;
                }
                Type::Array(a) => {
                    def.schema_type = SchemaType::Array;
                    if let Some(items) = &a.items {
                        def.items = Some(Box::new(resolve_and_transform_boxed_schema(items, spec)));
                    }
                }
                Type::Object(o) => {
                    def.schema_type = SchemaType::Object;
                    def.required = o.required.clone();
                    for (name, prop) in &o.properties {
                        let prop_schema = resolve_and_transform_boxed_schema(prop, spec);
                        def.properties.insert(name.clone(), prop_schema);
                    }
                    if let Some(ap) = &o.additional_properties {
                        match ap {
                            openapiv3::AdditionalProperties::Any(true) => {
                                def.additional_properties = Some(Box::new(SchemaDefinition::default()));
                            }
                            openapiv3::AdditionalProperties::Schema(s) => {
                                def.additional_properties =
                                    Some(Box::new(resolve_and_transform_schema(s, spec)));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        SchemaKind::OneOf { one_of } => {
            def.one_of = one_of
                .iter()
                .map(|s| resolve_and_transform_schema(s, spec))
                .collect();
        }
        SchemaKind::AnyOf { any_of } => {
            def.any_of = any_of
                .iter()
                .map(|s| resolve_and_transform_schema(s, spec))
                .collect();
        }
        SchemaKind::AllOf { all_of } => {
            def.all_of = all_of
                .iter()
                .map(|s| resolve_and_transform_schema(s, spec))
                .collect();
        }
        SchemaKind::Not { .. } => {
            // Not supported, treat as any
        }
        SchemaKind::Any(_) => {
            // Already defaults to Any
        }
    }

    def
}

/// Format a JSON value as a string.
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_openapi() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
  description: A test API
paths:
  /users:
    get:
      summary: List users
      responses:
        "200":
          description: Success
"#;
        let spec = parse_openapi(yaml).unwrap();
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
        assert_eq!(spec.operations.len(), 1);
        assert_eq!(spec.operations[0].method, HttpMethod::Get);
        assert_eq!(spec.operations[0].path, "/users");
    }

    #[test]
    fn test_parse_with_parameters() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /users/{id}:
    get:
      summary: Get user
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
        - name: include
          in: query
          schema:
            type: string
      responses:
        "200":
          description: Success
"#;
        let spec = parse_openapi(yaml).unwrap();
        assert_eq!(spec.operations[0].parameters.len(), 2);
        assert_eq!(spec.operations[0].parameters[0].name, "id");
        assert_eq!(spec.operations[0].parameters[0].location, ParameterLocation::Path);
        assert!(spec.operations[0].parameters[0].required);
    }

    #[test]
    fn test_parse_with_request_body() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
paths:
  /users:
    post:
      summary: Create user
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
      responses:
        "201":
          description: Created
"#;
        let spec = parse_openapi(yaml).unwrap();
        let rb = spec.operations[0].request_body.as_ref().unwrap();
        assert!(rb.required);
        assert_eq!(rb.content[0].media_type, "application/json");
    }

    #[test]
    fn test_http_method_badge_class() {
        assert_eq!(HttpMethod::Get.badge_class(), "badge-success");
        assert_eq!(HttpMethod::Post.badge_class(), "badge-primary");
        assert_eq!(HttpMethod::Delete.badge_class(), "badge-error");
    }
}

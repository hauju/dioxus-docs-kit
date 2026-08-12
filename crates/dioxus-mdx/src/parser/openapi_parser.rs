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
    let spec: OpenAPI = match serde_yaml::from_str(content) {
        Ok(s) => s,
        Err(yaml_err) => match serde_json::from_str(content) {
            Ok(s) => s,
            Err(json_err) => {
                // Report the error for the format the content most likely is,
                // so line/column info points at the real mistake.
                let msg = if content.trim_start().starts_with(['{', '[']) {
                    format!("JSON: {json_err}")
                } else {
                    format!("YAML: {yaml_err}")
                };
                return Err(OpenApiError::ParseError(msg));
            }
        },
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
                // Seed the cycle guard with this schema's own name so a direct
                // self-reference is caught on the first hop.
                let mut seen = vec![name.clone()];
                schemas.insert(name.clone(), transform_schema(schema, spec, &mut seen));
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
            operations.push(transform_operation(
                path,
                method,
                op,
                &item.parameters,
                spec,
            ));
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
fn transform_parameter(param_ref: &ReferenceOr<Parameter>, spec: &OpenAPI) -> Option<ApiParameter> {
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
        ParameterSchemaOrContent::Schema(s) => {
            Some(resolve_and_transform(s, spec, &mut Vec::new()))
        }
        _ => None,
    };

    Some(ApiParameter {
        name: data.name.clone(),
        location,
        description: data.description.clone(),
        required: data.required,
        deprecated: data.deprecated.unwrap_or(false),
        schema,
        example: data.example.as_ref().map(format_json_value),
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
            schema: media
                .schema
                .as_ref()
                .map(|s| resolve_and_transform(s, spec, &mut Vec::new())),
            example: media.example.as_ref().map(format_json_value),
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
                schema: media
                    .schema
                    .as_ref()
                    .map(|s| resolve_and_transform(s, spec, &mut Vec::new())),
                example: media.example.as_ref().map(format_json_value),
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

/// Lets one resolver serve both `ReferenceOr<Schema>` and `ReferenceOr<Box<Schema>>`.
trait AsSchema {
    fn as_schema(&self) -> &Schema;
}

impl AsSchema for Schema {
    fn as_schema(&self) -> &Schema {
        self
    }
}

impl AsSchema for Box<Schema> {
    fn as_schema(&self) -> &Schema {
        self
    }
}

/// Resolve a schema reference and transform it.
///
/// `seen` holds the component names currently being expanded. A reference back
/// into that set resolves to a name-only stub, so a self-referential schema
/// (`Node.children -> [Node]`) terminates instead of recursing until the stack
/// overflows. Accepts both `Schema` and `Box<Schema>` references.
fn resolve_and_transform<S: AsSchema>(
    schema_ref: &ReferenceOr<S>,
    spec: &OpenAPI,
    seen: &mut Vec<String>,
) -> SchemaDefinition {
    match schema_ref {
        ReferenceOr::Item(schema) => transform_schema(schema.as_schema(), spec, seen),
        ReferenceOr::Reference { reference } => {
            // Extract the reference name
            let ref_name = reference
                .strip_prefix("#/components/schemas/")
                .map(|s| s.to_string());

            // Already expanding this schema further up the stack - stop here
            if let Some(name) = &ref_name
                && seen.contains(name)
            {
                return SchemaDefinition {
                    ref_name: ref_name.clone(),
                    ..Default::default()
                };
            }

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
                if let Some(name) = &ref_name {
                    seen.push(name.clone());
                }
                let mut def = transform_schema(schema, spec, seen);
                if ref_name.is_some() {
                    seen.pop();
                }
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
fn transform_schema(schema: &Schema, spec: &OpenAPI, seen: &mut Vec<String>) -> SchemaDefinition {
    let mut def = SchemaDefinition {
        description: schema.schema_data.description.clone(),
        example: schema.schema_data.example.as_ref().map(format_json_value),
        default: schema.schema_data.default.as_ref().map(format_json_value),
        nullable: schema.schema_data.nullable,
        ..Default::default()
    };

    match &schema.schema_kind {
        SchemaKind::Type(t) => match t {
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
                    def.items = Some(Box::new(resolve_and_transform(items, spec, seen)));
                }
            }
            Type::Object(o) => {
                def.schema_type = SchemaType::Object;
                def.required = o.required.clone();
                for (name, prop) in &o.properties {
                    let prop_schema = resolve_and_transform(prop, spec, seen);
                    def.properties.insert(name.clone(), prop_schema);
                }
                if let Some(ap) = &o.additional_properties {
                    match ap {
                        openapiv3::AdditionalProperties::Any(true) => {
                            def.additional_properties = Some(Box::new(SchemaDefinition::default()));
                        }
                        openapiv3::AdditionalProperties::Schema(s) => {
                            def.additional_properties =
                                Some(Box::new(resolve_and_transform(s, spec, seen)));
                        }
                        _ => {}
                    }
                }
            }
        },
        SchemaKind::OneOf { one_of } => {
            def.one_of = one_of
                .iter()
                .map(|s| resolve_and_transform(s, spec, seen))
                .collect();
        }
        SchemaKind::AnyOf { any_of } => {
            def.any_of = any_of
                .iter()
                .map(|s| resolve_and_transform(s, spec, seen))
                .collect();
        }
        SchemaKind::AllOf { all_of } => {
            def.all_of = all_of
                .iter()
                .map(|s| resolve_and_transform(s, spec, seen))
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
    fn test_self_referential_schema_terminates() {
        let yaml = r##"
openapi: "3.0.0"
info:
  title: Tree API
  version: "1.0.0"
paths: {}
components:
  schemas:
    Node:
      type: object
      properties:
        name:
          type: string
        children:
          type: array
          items:
            $ref: "#/components/schemas/Node"
"##;
        let spec = parse_openapi(yaml).unwrap();
        let node = spec.schemas.get("Node").expect("Node schema");

        // The recursive branch resolves to a name-only stub rather than
        // expanding forever.
        let children = node.properties.get("children").expect("children property");
        let item = children.items.as_ref().expect("array items");
        assert_eq!(item.ref_name.as_deref(), Some("Node"));
        assert!(item.properties.is_empty());

        // The non-recursive branch is still expanded normally.
        assert_eq!(
            node.properties.get("name").map(|p| p.schema_type.clone()),
            Some(SchemaType::String)
        );
    }

    #[test]
    fn test_mutually_recursive_schemas_terminate() {
        let yaml = r##"
openapi: "3.0.0"
info:
  title: Loop API
  version: "1.0.0"
paths: {}
components:
  schemas:
    A:
      type: object
      properties:
        b:
          $ref: "#/components/schemas/B"
    B:
      type: object
      properties:
        a:
          $ref: "#/components/schemas/A"
"##;
        let spec = parse_openapi(yaml).unwrap();
        let a = spec.schemas.get("A").expect("A schema");
        let b_prop = a.properties.get("b").expect("b property");
        assert_eq!(b_prop.ref_name.as_deref(), Some("B"));

        // B was expanded once; its back-reference to A is the stub.
        let a_prop = b_prop.properties.get("a").expect("a property");
        assert_eq!(a_prop.ref_name.as_deref(), Some("A"));
        assert!(a_prop.properties.is_empty());
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
        assert_eq!(
            spec.operations[0].parameters[0].location,
            ParameterLocation::Path
        );
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
        assert_eq!(HttpMethod::Get.badge_class(), "badge-soft badge-success");
        assert_eq!(HttpMethod::Post.badge_class(), "badge-soft badge-primary");
        assert_eq!(HttpMethod::Delete.badge_class(), "badge-soft badge-error");
    }
}

//! Internal type definitions for parsed OpenAPI specifications.
//!
//! These types provide a simplified view of OpenAPI specs for rendering.

use serde_json::json;
use std::collections::BTreeMap;

/// Parsed OpenAPI specification.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenApiSpec {
    /// API info (title, version, description).
    pub info: ApiInfo,
    /// Server URLs.
    pub servers: Vec<ApiServer>,
    /// API operations grouped by tag.
    pub operations: Vec<ApiOperation>,
    /// Unique tags in order of appearance.
    pub tags: Vec<ApiTag>,
    /// Reusable schema definitions.
    pub schemas: BTreeMap<String, SchemaDefinition>,
}

/// API metadata.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ApiInfo {
    /// API title.
    pub title: String,
    /// API version.
    pub version: String,
    /// API description.
    pub description: Option<String>,
}

/// Server configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiServer {
    /// Server URL.
    pub url: String,
    /// Server description.
    pub description: Option<String>,
}

/// Tag metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiTag {
    /// Tag name.
    pub name: String,
    /// Tag description.
    pub description: Option<String>,
}

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "get" => Some(Self::Get),
            "post" => Some(Self::Post),
            "put" => Some(Self::Put),
            "delete" => Some(Self::Delete),
            "patch" => Some(Self::Patch),
            "head" => Some(Self::Head),
            "options" => Some(Self::Options),
            _ => None,
        }
    }

    /// Convert to uppercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }

    /// DaisyUI badge class for the method.
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Get => "badge-success",
            Self::Post => "badge-primary",
            Self::Put => "badge-warning",
            Self::Delete => "badge-error",
            Self::Patch => "badge-info",
            Self::Head => "badge-ghost",
            Self::Options => "badge-ghost",
        }
    }

    /// Tailwind background class for the method.
    pub fn bg_class(&self) -> &'static str {
        match self {
            Self::Get => "bg-success/10 border-success/30 text-success",
            Self::Post => "bg-primary/10 border-primary/30 text-primary",
            Self::Put => "bg-warning/10 border-warning/30 text-warning",
            Self::Delete => "bg-error/10 border-error/30 text-error",
            Self::Patch => "bg-info/10 border-info/30 text-info",
            Self::Head => "bg-base-300 border-base-content/20 text-base-content/70",
            Self::Options => "bg-base-300 border-base-content/20 text-base-content/70",
        }
    }
}

/// API endpoint operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiOperation {
    /// Unique operation ID.
    pub operation_id: Option<String>,
    /// HTTP method.
    pub method: HttpMethod,
    /// URL path.
    pub path: String,
    /// Short summary.
    pub summary: Option<String>,
    /// Full description.
    pub description: Option<String>,
    /// Tags for grouping.
    pub tags: Vec<String>,
    /// Parameters (path, query, header).
    pub parameters: Vec<ApiParameter>,
    /// Request body.
    pub request_body: Option<ApiRequestBody>,
    /// Response definitions.
    pub responses: Vec<ApiResponse>,
    /// Whether the endpoint is deprecated.
    pub deprecated: bool,
}

impl ApiOperation {
    /// Generate a URL-friendly slug for this operation.
    ///
    /// Uses `operation_id` if present (camelCase → kebab-case), otherwise
    /// falls back to `method-path` format.
    pub fn slug(&self) -> String {
        if let Some(op_id) = &self.operation_id {
            slugify_operation_id(op_id)
        } else {
            // Fallback: method-path format
            let path_slug = self
                .path
                .trim_matches('/')
                .replace('/', "-")
                .replace('{', "")
                .replace('}', "");
            format!("{}-{}", self.method.as_str().to_lowercase(), path_slug)
        }
    }

    /// Generate a curl command for this endpoint.
    pub fn generate_curl(&self, base_url: &str) -> String {
        let mut parts = vec!["curl".to_string()];

        // Method
        if !matches!(self.method, HttpMethod::Get) {
            parts.push(format!("-X {}", self.method.as_str()));
        }

        // Build URL with path params
        let mut url = format!("{}{}", base_url.trim_end_matches('/'), self.path);
        let mut query_parts = Vec::new();

        for param in &self.parameters {
            match param.location {
                ParameterLocation::Path => {
                    let placeholder = if let Some(schema) = &param.schema {
                        let val = schema.generate_example_json(0);
                        val.as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| val.to_string())
                    } else {
                        format!("{{{}}}", param.name)
                    };
                    url = url.replace(&format!("{{{}}}", param.name), &placeholder);
                }
                ParameterLocation::Query => {
                    if let Some(schema) = &param.schema {
                        let val = schema.generate_example_json(0);
                        let val_str = val
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| val.to_string());
                        query_parts.push(format!("{}={}", param.name, val_str));
                    }
                }
                _ => {}
            }
        }

        if !query_parts.is_empty() {
            url = format!("{}?{}", url, query_parts.join("&"));
        }

        parts.push(format!("\"{}\"", url));

        // Content-Type header if there's a request body
        if self.request_body.is_some() {
            parts.push("-H \"Content-Type: application/json\"".to_string());
        }

        // Request body
        if let Some(body) = &self.request_body {
            for content in &body.content {
                if content.media_type.contains("json") {
                    if let Some(schema) = &content.schema {
                        let example = schema.generate_example_json(0);
                        if let Ok(pretty) = serde_json::to_string_pretty(&example) {
                            parts.push(format!("-d '{}'", pretty));
                        }
                    }
                    break;
                }
            }
        }

        parts.join(" \\\n  ")
    }

    /// Generate a response example from the first 2xx response.
    ///
    /// Returns `Some((status_code, pretty_json))` if a 2xx response with
    /// content schema is found, `None` otherwise.
    pub fn generate_response_example(&self) -> Option<(String, String)> {
        for response in &self.responses {
            if response.status_code.starts_with('2') {
                for content in &response.content {
                    if let Some(schema) = &content.schema {
                        let example = schema.generate_example_json(0);
                        if let Ok(pretty) = serde_json::to_string_pretty(&example) {
                            return Some((response.status_code.clone(), pretty));
                        }
                    }
                }
            }
        }
        None
    }
}

/// Convert a camelCase operation ID to kebab-case slug.
fn slugify_operation_id(id: &str) -> String {
    let mut result = String::new();
    for (i, ch) in id.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    result
}

/// Parameter location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

impl ParameterLocation {
    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "path" => Some(Self::Path),
            "query" => Some(Self::Query),
            "header" => Some(Self::Header),
            "cookie" => Some(Self::Cookie),
            _ => None,
        }
    }

    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
            Self::Cookie => "cookie",
        }
    }

    /// Badge class for the location.
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Path => "badge-primary",
            Self::Query => "badge-info",
            Self::Header => "badge-warning",
            Self::Cookie => "badge-secondary",
        }
    }
}

/// API parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiParameter {
    /// Parameter name.
    pub name: String,
    /// Parameter location.
    pub location: ParameterLocation,
    /// Parameter description.
    pub description: Option<String>,
    /// Whether the parameter is required.
    pub required: bool,
    /// Whether the parameter is deprecated.
    pub deprecated: bool,
    /// Parameter schema.
    pub schema: Option<SchemaDefinition>,
    /// Example value.
    pub example: Option<String>,
}

/// Request body definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiRequestBody {
    /// Description.
    pub description: Option<String>,
    /// Whether the body is required.
    pub required: bool,
    /// Content by media type.
    pub content: Vec<MediaTypeContent>,
}

/// Content for a specific media type.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaTypeContent {
    /// Media type (e.g., "application/json").
    pub media_type: String,
    /// Schema for the content.
    pub schema: Option<SchemaDefinition>,
    /// Example value.
    pub example: Option<String>,
}

/// API response definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiResponse {
    /// HTTP status code or "default".
    pub status_code: String,
    /// Response description.
    pub description: String,
    /// Content by media type.
    pub content: Vec<MediaTypeContent>,
}

impl ApiResponse {
    /// Get badge class based on status code.
    pub fn status_badge_class(&self) -> &'static str {
        match self.status_code.chars().next() {
            Some('2') => "badge-success",
            Some('3') => "badge-info",
            Some('4') => "badge-warning",
            Some('5') => "badge-error",
            _ => "badge-ghost",
        }
    }
}

/// Schema type.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
    Any,
}

impl SchemaType {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Integer => "integer",
            Self::Boolean => "boolean",
            Self::Array => "array",
            Self::Object => "object",
            Self::Null => "null",
            Self::Any => "any",
        }
    }
}

/// Schema definition for a type.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDefinition {
    /// Schema type.
    pub schema_type: SchemaType,
    /// Format (e.g., "int64", "email", "date-time").
    pub format: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// For arrays, the item schema.
    pub items: Option<Box<SchemaDefinition>>,
    /// For objects, property schemas.
    pub properties: BTreeMap<String, SchemaDefinition>,
    /// Required property names.
    pub required: Vec<String>,
    /// Reference name (for $ref).
    pub ref_name: Option<String>,
    /// Enum values.
    pub enum_values: Vec<String>,
    /// Example value.
    pub example: Option<String>,
    /// Default value.
    pub default: Option<String>,
    /// Nullable flag.
    pub nullable: bool,
    /// Additional properties schema (for objects).
    pub additional_properties: Option<Box<SchemaDefinition>>,
    /// OneOf schemas.
    pub one_of: Vec<SchemaDefinition>,
    /// AnyOf schemas.
    pub any_of: Vec<SchemaDefinition>,
    /// AllOf schemas.
    pub all_of: Vec<SchemaDefinition>,
}

impl Default for SchemaDefinition {
    fn default() -> Self {
        Self {
            schema_type: SchemaType::Any,
            format: None,
            description: None,
            items: None,
            properties: BTreeMap::new(),
            required: Vec::new(),
            ref_name: None,
            enum_values: Vec::new(),
            example: None,
            default: None,
            nullable: false,
            additional_properties: None,
            one_of: Vec::new(),
            any_of: Vec::new(),
            all_of: Vec::new(),
        }
    }
}

impl SchemaDefinition {
    /// Get a display type string (e.g., "string", "array`<User>`", "object").
    pub fn display_type(&self) -> String {
        if let Some(ref_name) = &self.ref_name {
            return ref_name.clone();
        }

        match &self.schema_type {
            SchemaType::Array => {
                if let Some(items) = &self.items {
                    format!("array<{}>", items.display_type())
                } else {
                    "array".to_string()
                }
            }
            SchemaType::Object if !self.properties.is_empty() => "object".to_string(),
            other => {
                let mut s = other.as_str().to_string();
                if let Some(format) = &self.format {
                    s.push_str(&format!(" ({format})"));
                }
                s
            }
        }
    }

    /// Check if this is a complex type (object or array with object items).
    pub fn is_complex(&self) -> bool {
        matches!(self.schema_type, SchemaType::Object | SchemaType::Array)
            || !self.one_of.is_empty()
            || !self.any_of.is_empty()
            || !self.all_of.is_empty()
    }

    /// Generate example JSON for this schema.
    ///
    /// Uses explicit `example` if present, otherwise generates placeholder values by type.
    /// `depth` prevents infinite recursion from circular refs (max 5).
    pub fn generate_example_json(&self, depth: usize) -> serde_json::Value {
        if depth > 5 {
            return json!({});
        }

        // Use explicit example if available
        if let Some(example) = &self.example {
            if let Ok(val) = serde_json::from_str(example) {
                return val;
            }
            return json!(example);
        }

        match &self.schema_type {
            SchemaType::String => {
                if !self.enum_values.is_empty() {
                    return json!(self.enum_values[0]);
                }
                match self.format.as_deref() {
                    Some("uuid") => json!("550e8400-e29b-41d4-a716-446655440000"),
                    Some("date-time") => json!("2024-01-15T09:30:00Z"),
                    Some("date") => json!("2024-01-15"),
                    Some("uri") | Some("url") => json!("https://example.com"),
                    Some("email") => json!("user@example.com"),
                    _ => json!("string"),
                }
            }
            SchemaType::Integer => {
                if let Some(default) = &self.default {
                    if let Ok(n) = default.parse::<i64>() {
                        return json!(n);
                    }
                }
                json!(0)
            }
            SchemaType::Number => json!(0.0),
            SchemaType::Boolean => json!(true),
            SchemaType::Array => {
                if let Some(items) = &self.items {
                    json!([items.generate_example_json(depth + 1)])
                } else {
                    json!([])
                }
            }
            SchemaType::Object => {
                if self.properties.is_empty() {
                    return json!({});
                }
                let mut map = serde_json::Map::new();
                for (name, prop) in &self.properties {
                    map.insert(name.clone(), prop.generate_example_json(depth + 1));
                }
                serde_json::Value::Object(map)
            }
            SchemaType::Null => json!(null),
            SchemaType::Any => json!("any"),
        }
    }
}

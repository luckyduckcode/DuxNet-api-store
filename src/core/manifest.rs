use crate::core::data_structures::ServiceManifest;
use anyhow::{Context, Result};
use jsonschema::JSONSchema;
use serde_json::Value;
use serde_yaml;
use std::sync::LazyLock;

static MANIFEST_SCHEMA: LazyLock<JSONSchema> = LazyLock::new(|| {
    let schema_json = ManifestValidator::get_manifest_schema();
    JSONSchema::compile(&schema_json).expect("Failed to compile manifest schema")
});

/// Validates YAML service manifests against schema and business rules
pub struct ManifestValidator;

impl ManifestValidator {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    /// Parse and validate a YAML manifest
    pub fn validate_manifest(&self, yaml_content: &str) -> Result<ServiceManifest> {
        // Parse YAML
        let manifest: ServiceManifest = serde_yaml::from_str(yaml_content)
            .context("Failed to parse YAML manifest")?;
        
        // Convert to JSON for schema validation
        let json_value = serde_json::to_value(&manifest)
            .context("Failed to convert manifest to JSON")?;
        
        // Validate against schema
        if let Err(errors) = MANIFEST_SCHEMA.validate(&json_value) {
            let error_messages: Vec<String> = errors
                .map(|e| format!("Field '{}': {}", e.instance_path, e))
                .collect();
            anyhow::bail!("Validation errors: {}", error_messages.join(", "));
        }
        
        // Additional business rule validations
        self.validate_business_rules(&manifest)?;
        
        Ok(manifest)
    }
    
    /// Validate business rules beyond schema
    fn validate_business_rules(&self, manifest: &ServiceManifest) -> Result<()> {
        // Validate version format (semantic versioning)
        if !Self::is_valid_semver(&manifest.version) {
            anyhow::bail!("Version must follow semantic versioning (e.g., 1.0.0)");
        }
        
        // Validate service name (alphanumeric + hyphens only)
        if !manifest.name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            anyhow::bail!("Service name can only contain alphanumeric characters and hyphens");
        }
        
        // Validate container image format
        if !Self::is_valid_docker_image(&manifest.container.image) {
            anyhow::bail!("Invalid Docker image format");
        }
        
        // Validate port ranges
        for &port in &manifest.container.ports {
            if port == 0 || port > 65535 {
                anyhow::bail!("Port {} is out of valid range (1-65535)", port);
            }
        }
        
        // Validate API endpoints
        for endpoint in &manifest.api.endpoints {
            if !endpoint.path.starts_with('/') {
                anyhow::bail!("API endpoint path '{}' must start with '/'", endpoint.path);
            }
            
            let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
            if !valid_methods.contains(&endpoint.method.as_str()) {
                anyhow::bail!("Invalid HTTP method: {}", endpoint.method);
            }
        }
        
        // Validate SLA values
        if manifest.sla.uptime < 0.0 || manifest.sla.uptime > 100.0 {
            anyhow::bail!("SLA uptime must be between 0 and 100 percent");
        }
        
        Ok(())
    }
    
    fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
    
    fn is_valid_docker_image(image: &str) -> bool {
        // Basic validation: must contain at least name and tag
        image.contains(':') && !image.is_empty() && !image.starts_with(':')
    }
    
    /// JSON Schema for service manifests
    fn get_manifest_schema() -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["name", "version", "description", "category", "author", "container", "api", "pricing", "sla"],
            "properties": {
                "name": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "minLength": 3,
                    "maxLength": 64
                },
                "version": {
                    "type": "string",
                    "pattern": "^\\d+\\.\\d+\\.\\d+$"
                },
                "description": {
                    "type": "string",
                    "minLength": 10,
                    "maxLength": 1000
                },
                "category": {
                    "type": "string",
                    "enum": [
                        "ai", "blockchain", "data", "financial", "gaming", 
                        "iot", "media", "productivity", "security", "social", 
                        "utilities", "development", "analytics", "communication", "other"
                    ]
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "maxItems": 10
                },
                "author": {
                    "type": "object",
                    "required": ["name", "did"],
                    "properties": {
                        "name": { "type": "string", "minLength": 1 },
                        "email": { 
                            "anyOf": [
                                { "type": "string", "format": "email" },
                                { "type": "null" }
                            ]
                        },
                        "did": { "type": "string", "minLength": 10 },
                        "contact": { 
                            "anyOf": [
                                { "type": "string" },
                                { "type": "null" }
                            ]
                        }
                    }
                },
                "container": {
                    "type": "object",
                    "required": ["image", "ports", "resources"],
                    "properties": {
                        "image": { "type": "string", "minLength": 1 },
                        "ports": {
                            "type": "array",
                            "items": { "type": "integer", "minimum": 1, "maximum": 65535 },
                            "minItems": 1
                        },
                        "env": {
                            "type": "object",
                            "additionalProperties": { "type": "string" }
                        },
                        "resources": {
                            "type": "object",
                            "required": ["cpu", "memory"],
                            "properties": {
                                "cpu": { "type": "string" },
                                "memory": { "type": "string" },
                                "gpu": { 
                                    "anyOf": [
                                        { "type": "string" },
                                        { "type": "null" }
                                    ]
                                }
                            }
                        }
                    }
                },
                "api": {
                    "type": "object",
                    "required": ["openapi", "endpoints"],
                    "properties": {
                        "openapi": { "type": "string" },
                        "base_path": { "type": "string" },
                        "endpoints": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["path", "method", "description", "pricing"],
                                "properties": {
                                    "path": { "type": "string", "pattern": "^/" },
                                    "method": { 
                                        "type": "string", 
                                        "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                                    },
                                    "description": { "type": "string", "minLength": 1 },
                                    "pricing": {
                                        "type": "object",
                                        "required": ["model", "amount", "currency"],
                                        "properties": {
                                            "model": { 
                                                "type": "string", 
                                                "enum": ["per-call", "subscription", "free", "usage-based"]
                                            },
                                            "amount": { "type": "number", "minimum": 0 },
                                            "currency": { "type": "string", "minLength": 1 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "pricing": {
                    "type": "string",
                    "enum": ["per-call", "subscription", "free", "usage-based"]
                },
                "sla": {
                    "type": "object",
                    "required": ["uptime", "response_time_ms", "throughput_rps"],
                    "properties": {
                        "uptime": { "type": "number", "minimum": 0, "maximum": 100 },
                        "response_time_ms": { "type": "integer", "minimum": 1 },
                        "throughput_rps": { "type": "integer", "minimum": 1 }
                    }
                },
                "reputation": {
                    "oneOf": [
                        { "type": "null" },
                        {
                            "type": "object",
                            "properties": {
                                "score": { "type": "number", "minimum": 0, "maximum": 5 },
                                "total_calls": { "type": "integer", "minimum": 0 },
                                "reviews": { "type": "integer", "minimum": 0 },
                                "uptime_actual": { "type": "number", "minimum": 0, "maximum": 100 }
                            }
                        }
                    ]
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_manifest() {
        let yaml_content = r#"
name: "test-service"
version: "1.0.0"
description: "A test service for validation"
category: "ai"
tags: ["test", "ai"]
author:
  name: "Test Author"
  did: "did:dux:test123"
container:
  image: "test/service:1.0.0"
  ports: [8080]
  env:
    PORT: "8080"
  resources:
    cpu: "500m"
    memory: "1Gi"
api:
  openapi: "3.1.0"
  base_path: "/api/v1"
  endpoints:
    - path: "/test"
      method: "GET"
      description: "Test endpoint"
      pricing:
        model: "free"
        amount: 0.0
        currency: "DUX"
pricing: "free"
sla:
  uptime: 99.0
  response_time_ms: 100
  throughput_rps: 10
"#;
        
        let validator = ManifestValidator::new().unwrap();
        let result = validator.validate_manifest(yaml_content);
        if let Err(e) = &result {
            println!("Validation error: {:?}", e);
        }
        assert!(result.is_ok());
    }
}

pub mod core;
pub mod api;
pub mod network;
pub mod wallet;
pub mod container;
pub mod frontend;
pub mod gateway;
pub mod config;
pub mod monitoring;

#[cfg(test)]
mod tests {
    use crate::core::manifest::ManifestValidator;

    #[tokio::test]
    async fn test_ai_text_analyzer_manifest() {
        let yaml_content = r#"
name: "ai-text-analyzer"
version: "1.0.0"
description: "Advanced AI-powered text analysis service"
category: "ai"
tags: ["nlp", "sentiment", "entities", "ai", "text-analysis"]

author:
  name: "AI Solutions Corp"
  did: "did:dux:zQ3shwQPhT8p3Qd7h6QB8hkVxJ9EJ2nJ7KGGMHSqN8vP1R6Mm"
  email: "support@aisolutions.com"
  contact: "support@aisolutions.com"

container:
  image: "aisolutions/text-analyzer:1.0.0"
  ports: [8080]
  env:
    MODEL_TYPE: "transformer"
    MAX_BATCH_SIZE: "32"
  resources:
    cpu: "1000m"
    memory: "2Gi"
    gpu: "1x nvidia-tesla-t4"

api:
  openapi: "3.1.0"
  base_path: "/api/v1"
  endpoints:
    - path: "/analyze"
      method: "POST"
      description: "Analyze text for sentiment, entities, and topics"
      pricing:
        model: "per-call"
        amount: 0.05
        currency: "DUX"

pricing: "per-call"

sla:
  uptime: 99.9
  response_time_ms: 500
  throughput_rps: 100
"#;

        let validator = ManifestValidator::new().unwrap();
        let result = validator.validate_manifest(yaml_content);
        
        assert!(result.is_ok(), "Manifest validation should succeed: {:?}", result.err());
        
        let manifest = result.unwrap();
        assert_eq!(manifest.name, "ai-text-analyzer");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.category, "ai");
        assert_eq!(manifest.container.image, "aisolutions/text-analyzer:1.0.0");
        assert!(matches!(manifest.pricing, crate::core::data_structures::PricingModel::PerCall));
        
        println!("✅ AI Text Analyzer manifest validation passed!");
    }

    #[tokio::test]
    async fn test_manifest_validation_errors() {
        let invalid_yaml = r#"
name: "invalid-service"
version: "invalid-version"
description: "Test"
category: "ai"
"#;

        let validator = ManifestValidator::new().unwrap();
        let result = validator.validate_manifest(invalid_yaml);
        
        assert!(result.is_err(), "Invalid manifest should fail validation");
        println!("✅ Invalid manifest correctly rejected");
    }
}

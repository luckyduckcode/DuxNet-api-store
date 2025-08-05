// Quick test to validate our YAML manifest implementation
use duxnet::core::manifest::ManifestValidator;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test the YAML manifest validation
    let yaml_content = fs::read_to_string("examples/ai-text-analyzer.yml")?;
    
    let validator = ManifestValidator::new()?;
    
    match validator.validate_manifest(&yaml_content) {
        Ok(manifest) => {
            println!("✅ Manifest validation successful!");
            println!("Service: {} v{}", manifest.name, manifest.version);
            println!("Description: {}", manifest.description);
            println!("Container: {}", manifest.container.image);
            println!("Price: {} DUX per request", manifest.pricing.per_request);
        }
        Err(e) => {
            println!("❌ Manifest validation failed: {}", e);
        }
    }
    
    Ok(())
}

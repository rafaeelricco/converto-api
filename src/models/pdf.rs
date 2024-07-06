use serde::Deserialize;

#[derive(Deserialize)]
pub enum CompressionLevel {
    Low,
    Medium,
    High,
}

impl From<&str> for CompressionLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => CompressionLevel::Low,
            "high" => CompressionLevel::High,
            _ => CompressionLevel::Medium,
        }
    }
}
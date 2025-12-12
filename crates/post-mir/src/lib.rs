mod error;
pub mod r#match;

pub use error::PatternAnalysisError;
pub use r#match::lower_matches;

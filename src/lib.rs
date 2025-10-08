// Library interface for build-it-agent
// This exposes modules for integration testing and potential library usage

pub mod types;
pub mod language;
pub mod rusq;

// Re-export commonly used types
pub use types::{
    TestCase, ExecuteRequest, ExecuteResponse, CaseResult, ExecutionStatus
};
pub use language::{LanguageConfig, LanguageInfo};
pub use rusq::{
    Priority, Message, RusqConfig, RusqMetrics, MpmcQueue, RusqError
};

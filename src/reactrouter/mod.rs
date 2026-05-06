pub mod builder;
pub mod detection;

pub use builder::{build_reactrouter_project, BuildResult, PackageManager};
pub use detection::{detect_reactrouter_project, validate_build_output, ReactRouterProjectInfo};

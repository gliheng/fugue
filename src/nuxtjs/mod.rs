pub mod builder;
pub mod detection;

pub use builder::{build_nuxt_project, BuildResult, PackageManager};
pub use detection::{detect_nuxt_project, validate_build_output, NuxtProjectInfo};

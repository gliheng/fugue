#![allow(dead_code)]

use crate::error::{FugueError, Result};

pub fn validate_app_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(FugueError::ValidationError(
            "App name cannot be empty".to_string(),
        ));
    }

    if name.len() > 64 {
        return Err(FugueError::ValidationError(
            "App name must be 64 characters or less".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(FugueError::ValidationError(
            "App name can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        ));
    }

    Ok(())
}

pub fn validate_code_size(code: &str) -> Result<()> {
    if code.is_empty() {
        return Err(FugueError::ValidationError(
            "Code cannot be empty".to_string(),
        ));
    }

    if code.len() > crate::config::defaults::MAX_FUNCTION_SIZE {
        return Err(FugueError::ValidationError(format!(
            "Code must be less than {} bytes",
            crate::config::defaults::MAX_FUNCTION_SIZE
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_app_name_valid() {
        assert!(validate_app_name("my-app").is_ok());
        assert!(validate_app_name("hello_world").is_ok());
        assert!(validate_app_name("app123").is_ok());
        assert!(validate_app_name("a").is_ok());
    }

    #[test]
    fn test_validate_app_name_empty() {
        let result = validate_app_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_app_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_app_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("64 characters"));
    }

    #[test]
    fn test_validate_app_name_max_length() {
        let max_name = "a".repeat(64);
        assert!(validate_app_name(&max_name).is_ok());
    }

    #[test]
    fn test_validate_app_name_invalid_chars() {
        assert!(validate_app_name("my app").is_err());
        assert!(validate_app_name("my.app").is_err());
        assert!(validate_app_name("my@app").is_err());
        assert!(validate_app_name("my/app").is_err());
    }

    #[test]
    fn test_validate_code_size_valid() {
        assert!(validate_code_size("console.log('hello')").is_ok());
    }

    #[test]
    fn test_validate_code_size_empty() {
        let result = validate_code_size("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }
}

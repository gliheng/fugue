use crate::error::{FugueError, Result};

pub fn validate_function_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(FugueError::ValidationError(
            "Function name cannot be empty".to_string(),
        ));
    }

    if name.len() > 64 {
        return Err(FugueError::ValidationError(
            "Function name must be 64 characters or less".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(FugueError::ValidationError(
            "Function name can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        ));
    }

    Ok(())
}

pub fn validate_function_code(code: &str) -> Result<()> {
    if code.is_empty() {
        return Err(FugueError::ValidationError(
            "Function code cannot be empty".to_string(),
        ));
    }

    if code.len() > crate::config::MAX_FUNCTION_SIZE {
        return Err(FugueError::ValidationError(format!(
            "Function code must be less than {} bytes",
            crate::config::MAX_FUNCTION_SIZE
        )));
    }

    Ok(())
}

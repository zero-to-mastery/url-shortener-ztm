//! # Validation Module
//!
//! This module provides validation utilities for custom aliases used in URL shortening.
//! It ensures aliases meet security and usability requirements.

use crate::errors::ApiError;
use regex::Regex;
use std::collections::HashSet;

/// Maximum allowed length for custom aliases
const MAX_ALIAS_LENGTH: usize = 50;

/// Minimum allowed length for custom aliases
const MIN_ALIAS_LENGTH: usize = 1;

/// Reserved aliases that cannot be used by users
const RESERVED_ALIASES: &[&str] = &[
    "admin",
    "api",
    "static",
    "health",
    "health_check",
    "login",
    "register",
    "dashboard",
    "profile",
    "logout",
    "shorten",
    "redirect",
    "users",
    "tags",
    "public",
    "help",
    "about",
    "contact",
    "terms",
    "privacy",
    "favicon.ico",
    "robots.txt",
    "sitemap.xml",
];

/// Validates a custom alias for URL shortening.
///
/// This function performs comprehensive validation on custom aliases to ensure
/// they are safe, usable, and meet the service requirements.
///
/// # Validation Rules
///
/// 1. **Length**: Must be between 1 and 50 characters
/// 2. **Characters**: Only alphanumeric characters, underscores, and hyphens allowed
/// 3. **Reserved words**: Cannot use reserved system aliases
/// 4. **Case sensitivity**: Aliases are case-sensitive
/// 5. **Empty/whitespace**: Cannot be empty or contain only whitespace
///
/// # Arguments
///
/// * `alias` - The custom alias to validate
///
/// # Returns
///
/// Returns `Ok(())` if the alias is valid, or `Err(ApiError::Unprocessable)`
/// with a descriptive error message if validation fails.
///
/// # Examples
///
/// ```rust
/// use url_shortener_ztm_lib::validation::validate_alias;
///
/// // Valid aliases
/// assert!(validate_alias("my-link").is_ok());
/// assert!(validate_alias("project_2024").is_ok());
/// assert!(validate_alias("ABC123").is_ok());
///
/// // Invalid aliases
/// assert!(validate_alias("admin").is_err()); // reserved word
/// assert!(validate_alias("").is_err()); // empty
/// assert!(validate_alias("my@link").is_err()); // invalid character
/// ```
pub fn validate_alias(alias: &str) -> Result<(), ApiError> {
    // Check if alias is empty or contains only whitespace
    if alias.trim().is_empty() {
        return Err(ApiError::Unprocessable(
            "Alias cannot be empty or contain only whitespace".to_string(),
        ));
    }

    // Check length constraints
    if alias.len() < MIN_ALIAS_LENGTH {
        return Err(ApiError::Unprocessable(format!(
            "Alias must be at least {} character long",
            MIN_ALIAS_LENGTH
        )));
    }

    if alias.len() > MAX_ALIAS_LENGTH {
        return Err(ApiError::Unprocessable(format!(
            "Alias cannot exceed {} characters",
            MAX_ALIAS_LENGTH
        )));
    }

    // Check for reserved aliases (case-insensitive)
    let alias_lower = alias.to_lowercase();
    if RESERVED_ALIASES.contains(&alias_lower.as_str()) {
        return Err(ApiError::Unprocessable(format!(
            "Alias '{}' is reserved and cannot be used",
            alias
        )));
    }

    // Validate character set using regex
    let valid_chars_pattern =
        Regex::new(r"^[A-Za-z0-9_-]+$").expect("Failed to compile alias validation regex");

    if !valid_chars_pattern.is_match(alias) {
        return Err(ApiError::Unprocessable(
            "Alias can only contain letters (A-Z, a-z), numbers (0-9), underscores (_), and hyphens (-)".to_string(),
        ));
    }

    // Check for consecutive special characters (optional enhancement)
    if alias.contains("__") || alias.contains("--") || alias.contains("_-") || alias.contains("-_")
    {
        return Err(ApiError::Unprocessable(
            "Alias cannot contain consecutive special characters".to_string(),
        ));
    }

    // Check that alias doesn't start or end with special characters
    if alias.starts_with('_')
        || alias.starts_with('-')
        || alias.ends_with('_')
        || alias.ends_with('-')
    {
        return Err(ApiError::Unprocessable(
            "Alias cannot start or end with underscore or hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Checks if an alias is available by validating it against a set of existing aliases.
///
/// This function is used to check if a custom alias is already in use before
/// attempting to create a new shortened URL.
///
/// # Arguments
///
/// * `alias` - The alias to check for availability
/// * `existing_aliases` - Set of existing aliases to check against
///
/// # Returns
///
/// Returns `Ok(())` if the alias is available, or `Err(ApiError::Unprocessable)`
/// if the alias is already in use.
///
/// # Examples
///
/// ```rust
/// use url_shortener_ztm_lib::validation::check_alias_availability;
/// use std::collections::HashSet;
///
/// let mut existing = HashSet::new();
/// existing.insert("existing-link".to_string());
///
/// assert!(check_alias_availability("new-link", &existing).is_ok());
/// assert!(check_alias_availability("existing-link", &existing).is_err());
/// ```
pub fn check_alias_availability(
    alias: &str,
    existing_aliases: &HashSet<String>,
) -> Result<(), ApiError> {
    if existing_aliases.contains(alias) {
        return Err(ApiError::Unprocessable(format!(
            "Alias '{}' is already in use",
            alias
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_valid_aliases() {
        let max_length_alias = "a".repeat(MAX_ALIAS_LENGTH);
        let valid_aliases = vec![
            "my-link",
            "project_2024",
            "ABC123",
            "test123",
            "my_awesome_link",
            "link-2024",
            "a",               // minimum length
            &max_length_alias, // maximum length
        ];

        for alias in valid_aliases {
            assert!(
                validate_alias(alias).is_ok(),
                "Alias '{}' should be valid",
                alias
            );
        }
    }

    #[test]
    fn test_invalid_aliases() {
        let too_long_alias = "a".repeat(MAX_ALIAS_LENGTH + 1);
        let invalid_aliases = vec![
            ("", "empty alias"),
            ("   ", "whitespace only"),
            ("admin", "reserved word"),
            ("API", "reserved word (case insensitive)"),
            ("my@link", "invalid character @"),
            ("my link", "space character"),
            ("my.link", "invalid character ."),
            ("my+link", "invalid character +"),
            ("__test", "starts with underscore"),
            ("test__", "ends with underscore"),
            ("--test", "starts with hyphen"),
            ("test--", "ends with hyphen"),
            ("test__link", "consecutive underscores"),
            ("test--link", "consecutive hyphens"),
            ("test_-link", "consecutive special chars"),
            ("test-_link", "consecutive special chars"),
            (&too_long_alias, "too long"),
        ];

        for (alias, reason) in invalid_aliases {
            assert!(
                validate_alias(alias).is_err(),
                "Alias '{}' should be invalid because: {}",
                alias,
                reason
            );
        }
    }

    #[test]
    fn test_reserved_aliases() {
        for reserved in RESERVED_ALIASES {
            // Test lowercase
            assert!(
                validate_alias(reserved).is_err(),
                "Lowercase '{}' should be reserved",
                reserved
            );
            // Test uppercase
            assert!(
                validate_alias(&reserved.to_uppercase()).is_err(),
                "Uppercase '{}' should be reserved",
                reserved
            );
            // Test mixed case
            let mixed = reserved
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i % 2 == 0 {
                        c.to_uppercase().collect::<String>()
                    } else {
                        c.to_lowercase().collect::<String>()
                    }
                })
                .collect::<String>();
            assert!(
                validate_alias(&mixed).is_err(),
                "Mixed case '{}' should be reserved",
                mixed
            );
        }
    }

    #[test]
    fn test_alias_availability() {
        let mut existing = HashSet::new();
        existing.insert("existing-link".to_string());
        existing.insert("another-link".to_string());

        // Available aliases
        assert!(check_alias_availability("new-link", &existing).is_ok());
        assert!(check_alias_availability("different-alias", &existing).is_ok());

        // Unavailable aliases
        assert!(check_alias_availability("existing-link", &existing).is_err());
        assert!(check_alias_availability("another-link", &existing).is_err());
    }
}

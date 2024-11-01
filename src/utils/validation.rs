use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use validator::ValidationError;

lazy_static! {
    static ref NUMERIC_ONLY: Regex = Regex::new(r"^\d+$").unwrap();
}

pub fn validate_email_verification_code_length(code: &str) -> Result<(), ValidationError> {
    if code.len() != 6 {
        let mut error = ValidationError::new("invalid_length");
        error.message = Some(Cow::from("The code must be exactly 6 digits long"));
        return Err(error);
    }

    Ok(())
}

pub fn validate_email_verification_code_format(code: &str) -> Result<(), ValidationError> {
    if !NUMERIC_ONLY.is_match(code) {
        let mut error = ValidationError::new("invalid_format");
        error.message = Some(Cow::from("The code must contain only numbers"));
        return Err(error);
    }

    Ok(())
}

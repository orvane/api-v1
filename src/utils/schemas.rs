pub const USER_SCHEMA: &str = r#"
    DEFINE TABLE user SCHEMAFULL;

    DEFINE FIELD email ON TABLE user TYPE string;
    DEFINE FIELD email_verified ON TABLE user TYPE bool DEFAULT false;
    DEFINE FIELD password_hash ON TABLE user TYPE string;
    DEFINE FIELD created_at ON TABLE user TYPE datetime;
"#;

pub const EMAIL_VERIFICATION_SCHEMA: &str = r#"
    DEFINE TABLE email_verification SCHEMAFULL;

    DEFINE FIELD code ON TABLE email_verification TYPE string;
    DEFINE FIELD created_at ON TABLE email_verification TYPE datetime;
    DEFINE FIELD expires_at ON TABLE email_verification TYPE datetime;

    DEFINE FIELD user ON TABLE email_verification TYPE record<user>;
"#;

pub const SESSION_SCHEMA: &str = r#"
    DEFINE TABLE session SCHEMAFULL;

    DEFINE FIELD authorized ON TABLE session TYPE bool;
    DEFINE FIELD created_at ON TABLE session TYPE datetime;
    DEFINE FIELD expires_at ON TABLE session TYPE datetime;
    DEFINE FIELD last_accessed_at ON TABLE session TYPE datetime;

    DEFINE FIELD user ON TABLE session TYPE record<user>;
"#;

pub const PASSWORD_RESET_REQUEST_SCHEMA: &str = r#"
    DEFINE TABLE password_reset_request SCHEMAFULL;

    DEFINE FIELD created_at ON TABLE password_reset_request TYPE datetime;
    DEFINE FIELD expires_at ON TABLE password_reset_request TYPE datetime;

    DEFINE FIELD user ON TABLE password_reset_request TYPE record<user>;
"#;

// TODO: Create schemas for relation tables

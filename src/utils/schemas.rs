pub const USER_SCHEMA: &str = r#"
    DEFINE TABLE user SCHEMAFULL;

    DEFINE FIELD email ON TABLE user TYPE string;
    DEFINE FIELD email_verified ON TABLE user TYPE bool;
    DEFINE FIELD password_hash ON TABLE user TYPE string;
    DEFINE FIELD created_at ON TABLE user TYPE datetime;
"#;

pub const EMAIL_VERIFICATION_SCHEMA: &str = r#"
    DEFINE TABLE email_verification SCHEMAFULL;

    DEFINE FIELD code ON TABLE email_verification TYPE string;
    DEFINE FIELD created_at ON TABLE email_verification TYPE datetime;
    DEFINE FIELD expired_at ON TABLE email_verification TYPE datetime;
"#;

pub const SESSION_SCHEMA: &str = r#"
    DEFINE TABLE session SCHEMAFULL;

    DEFINE FIELD authorized ON TABLE session TYPE bool ;
    DEFINE FIELD created_at ON TABLE session TYPE datetime;
    DEFINE FIELD expired_at ON TABLE session TYPE datetime;
    DEFINE FIELD last_accessed_at_at ON TABLE session TYPE datetime;
"#;

// TODO: Create schemas for relation tables

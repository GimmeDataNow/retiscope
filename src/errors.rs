use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum RetiscopeError {
    FailedToParse,
    FailedToConnectToDB,
    FailedToConfigureDB,
    FailedToSignIn,
    FailedToSendQuery,
    FailedQuery,
    PlaceholderError,
}

impl fmt::Display for RetiscopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetiscopeError::FailedToParse => write!(f, "failed to parse"),
            RetiscopeError::FailedToConnectToDB => write!(f, "failed to parse"),
            RetiscopeError::FailedToConfigureDB => write!(f, "failed to parse"),
            RetiscopeError::FailedToSignIn => write!(f, "failed to parse"),
            RetiscopeError::FailedToSendQuery => write!(f, "failed to parse"),
            RetiscopeError::FailedQuery => write!(f, "failed to parse"),
            RetiscopeError::PlaceholderError => write!(f, "failed to parse"),
        }
    }
}

impl std::error::Error for RetiscopeError {}

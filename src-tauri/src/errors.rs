#[derive(Debug)]
pub enum RetiscopeError {
    FailedToParse,
    FailedToConnectToDB,
    FailedToConfigureDB,
    FailedToSignIn,
    FailedToSendQuery,
    FailedQuery,
}

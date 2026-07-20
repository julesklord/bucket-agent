use agent_client_protocol as acp;

use crate::auth::{AuthManager, BucketAuth};

/// Require first-party auth from a sync context, accepting tokens in the client-side buffer window.
pub(crate) fn require_first_party_auth(
    auth_manager: &AuthManager,
    missing_message: &'static str,
    non_first_party_message: &'static str,
) -> Result<BucketAuth, acp::Error> {
    let auth = auth_manager
        .current_or_expired()
        .ok_or_else(|| acp::Error::auth_required().data(missing_message))?;
    if !auth.is_first_party_auth() {
        return Err(acp::Error::auth_required().data(non_first_party_message));
    }
    Ok(auth)
}

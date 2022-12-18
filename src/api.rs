use super::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiErr {
    #[error(transparent)]
    RepoErr(#[from] RepoErr),
    #[error("Bad arguments: {0}")]
    ArgumentErr(String),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "function", content = "params")]
pub enum Payload {
    #[serde(rename = "fetchInvitation")]
    FetchInvitation { id: String },
    #[serde(rename = "updateInvitation")]
    UpdateInvitation { invitation: InvitationATO },
}

pub async fn route<T: InviteeRepo + RelationRepo>(
    params: Payload,
    db_service: T,
) -> Result<InvitationATO, ApiErr> {
    match params {
        Payload::FetchInvitation { id } => fetch_invitation(&id, db_service).await,
        Payload::UpdateInvitation { invitation } => {
            update_invitation(&invitation, db_service).await
        }
    }
}

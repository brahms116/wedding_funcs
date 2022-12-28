use super::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiErr {
    #[error(transparent)]
    RepoErr(#[from] RepoErr),
    #[error("Bad argument: {0}")]
    ArgumentErr(String),
}

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "function", content = "params")]
pub enum Payload {
    #[serde(rename = "fetchInvitation")]
    FetchInvitation { id: String },
    #[serde(rename = "updateInvitation")]
    UpdateInvitation { invitation: InvitationATO },
}

pub async fn handle_request<T: InviteeRepo + RelationRepo>(
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

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn payload_should_deserialize() {
        let json = json!({
            "function":"fetchInvitation",
            "params": {
                "id":"myid"
            }
        });

        let payload: Result<Payload, _> = serde_json::from_value(json);

        let payload = payload.expect("should parse properly");

        let correct = Payload::FetchInvitation {
            id: String::from("myid"),
        };

        assert_eq!(payload, correct);
    }
}

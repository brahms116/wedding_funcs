use super::*;
use async_trait::async_trait;
use thiserror::Error;
use tracing::{event, Level};

#[derive(Error, Debug)]
pub enum RepoErr {
    #[error("Item with id {0}, could not be found")]
    ItemNotFound(String),
    #[error("Oops, an error occured: {0}")]
    DBFailure(String),
}

#[async_trait]
pub trait InviteeRepo {
    async fn get_invitee_by_id(&self, id: &str) -> Result<InviteeDTO, RepoErr>;
    async fn get_invitee_by_ids(&self, ids: &Vec<&str>) -> Result<Vec<InviteeDTO>, RepoErr>;
    async fn update_invitee(&self, invitee: &UpdateInviteeParams) -> Result<InviteeDTO, RepoErr>;
}

#[async_trait]
pub trait RelationRepo {
    async fn get_dependents(&self, id: &str) -> Result<Vec<String>, RepoErr>;
}

#[tracing::instrument(skip(db))]
pub async fn fetch_invitation<T: InviteeRepo + RelationRepo>(
    id: &str,
    db: T,
) -> Result<InvitationATO, ApiErr> {
    let dependent_result = db.get_dependents(id).await.map_err(|e| ApiErr::RepoErr(e));

    if let Err(err) = dependent_result {
        event!(Level::ERROR, "Failed to find dependents");
        return Err(err);
    }
    let dependent_result = dependent_result.expect("Should handle err");

    let mut dependents: Option<Vec<InviteeDTO>> = None;
    if dependent_result.len() > 0 {
        let invitees = db
            .get_invitee_by_ids(&dependent_result.iter().map(|e| e.as_str()).collect())
            .await
            .map_err(|e| ApiErr::RepoErr(e));
        if let Err(err) = invitees {
            event!(Level::ERROR, "Failed to find dependents");
            return Err(err);
        }
        let invitees = invitees.expect("Should handle err");
        dependents = Some(invitees)
    }

    let primary_invitee = db
        .get_invitee_by_id(id)
        .await
        .map_err(|e| ApiErr::RepoErr(e));
    if let Err(err) = primary_invitee {
        event!(Level::ERROR, "Failed to find primary invitee");
        return Err(err);
    }
    let primary_invitee = primary_invitee.expect("Should handle err");

    Ok(InvitationATO {
        primary_invitee,
        dependents: dependents.unwrap_or(vec![]),
    })
}

#[tracing::instrument(skip(db))]
pub async fn update_invitation<T: InviteeRepo>(
    invitation: &InvitationATO,
    db: T,
) -> Result<InvitationATO, ApiErr> {
    let mut result = invitation.clone();
    let primary = UpdateInviteeParams::from(&invitation.primary_invitee);

    let primary = db
        .update_invitee(&primary)
        .await
        .map_err(|e| ApiErr::RepoErr(e));

    if let Err(err) = primary {
        event!(Level::ERROR, "Failed to update primary invitee");
        return Err(err);
    }
    let primary = primary.expect("Should handle err");

    result.primary_invitee = primary;

    if invitation.dependents.len() > 0 {
        let mut dependents_result: Vec<InviteeDTO> = vec![];

        for invitee in &invitation.dependents {
            let param = UpdateInviteeParams::from(invitee);
            let dependent_result = db
                .update_invitee(&param)
                .await
                .map_err(|e| ApiErr::RepoErr(e));
            if let Err(err) = dependent_result {
                event!(
                    Level::ERROR,
                    msg = "Failed to update dependent invitee",
                    ?invitee
                );
                return Err(err);
            }
            let dependent_result = dependent_result.expect("Should handle err");
            dependents_result.push(dependent_result);
        }

        result.dependents = dependents_result
    }

    Ok(result)
}

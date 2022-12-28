use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct InviteeDTO {
    pub id: String,
    pub fname: String,
    pub lname: String,
    pub rsvp: Option<bool>,
    pub dietary_requirements: String,
}

impl TryFrom<&Row> for InviteeDTO {
    type Error = &'static str;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let id: Result<String, _> = value.try_get(0);
        let fname: Result<String, _> = value.try_get(1);
        let lname: Result<String, _> = value.try_get(2);
        let rsvp: Result<String, _> = value.try_get(3);
        let dietary_requirements: Result<String, _> = value.try_get(4);

        let id = id.map_err(|_| "Could not convert id")?;
        let fname = fname.map_err(|_| "Could not convert fname")?;
        let lname = lname.map_err(|_| "Could not convert lname")?;
        let rsvp = rsvp.map_err(|_| "Could not convert rsvp")?;
        let dietary_requirements =
            dietary_requirements.map_err(|_| "Could not convert dietary_requirements")?;

        let rsvp = match rsvp.as_str() {
            "NotComing" => Some(false),
            "Coming" => Some(true),
            _ => None,
        };

        Ok(Self {
            id,
            fname,
            lname,
            rsvp,
            dietary_requirements,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct InvitationATO {
    pub primary_invitee: InviteeDTO,
    pub dependents: Vec<InviteeDTO>,
}

pub struct UpdateInviteeParams {
    pub id: String,
    pub rsvp: Option<bool>,
    pub dietary_requirements: String,
}

impl From<&InviteeDTO> for UpdateInviteeParams {
    fn from(a: &InviteeDTO) -> Self {
        Self {
            id: a.id.clone(),
            rsvp: a.rsvp.clone(),
            dietary_requirements: a.dietary_requirements.clone(),
        }
    }
}

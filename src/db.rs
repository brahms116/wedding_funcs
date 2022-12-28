use super::*;
use async_trait::async_trait;
use tokio_postgres::Client;
use tracing::{event, Level};

pub struct DB<'a> {
    pub client: &'a Client,
}

#[async_trait]
impl<'a> RelationRepo for DB<'a> {
    async fn get_dependents(&self, id: &str) -> Result<Vec<String>, RepoErr> {
        let result = self
            .client
            .query("SELECT child FROM relation WHERE parent = $1::TEXT", &[&id])
            .await
            .map_err(|e| RepoErr::DBFailure(e.to_string()))?;

        let result: Result<Vec<String>, _> = result.iter().map(|e| e.try_get(0)).collect();
        result.map_err(|e| RepoErr::DBFailure(e.to_string()))
    }
}

#[async_trait]
impl<'a> InviteeRepo for DB<'a> {
    #[tracing::instrument(skip(self))]
    async fn get_invitee_by_id(&self, id: &str) -> Result<InviteeDTO, RepoErr> {
        let result = self.client.query(
            "SELECT id, fname, lname, rsvp, dietary_requirements FROM invitee WHERE id = $1::TEXT",
            &[&id],
        ).await;

        if let Err(err) = result {
            event!(Level::ERROR, "Failed to run find invitee query");
            return Err(RepoErr::DBFailure(err.to_string()));
        }
        let result = result.expect("Should handle err");

        if let None = result.get(0) {
            return Err(RepoErr::ItemNotFound(id.to_string()));
        }

        let result = result.get(0).unwrap();

        let update_result = self
            .client
            .query(
                "UPDATE invitee SET invitation_opened = true WHERE id = $1::TEXT",
                &[&id],
            )
            .await;

        if let Err(err) = update_result {
            event!(
                Level::ERROR,
                "Failed to update invitation_opened status for invitee"
            );
            return Err(RepoErr::DBFailure(err.to_string()));
        }

        let invitee = InviteeDTO::try_from(result).map_err(|e| RepoErr::DBFailure(e.to_string()));

        if let Err(err) = invitee {
            event!(
                Level::ERROR,
                msg = "Failed to parse db row into invitee",
                ?result
            );
            return Err(err);
        }

        Ok(invitee.expect("Should handle err"))
    }

    #[tracing::instrument(skip(self))]
    async fn get_invitee_by_ids(&self, ids: &Vec<&str>) -> Result<Vec<InviteeDTO>, RepoErr> {
        let result = self
            .client
            .query(
                "SELECT id, fname, lname, rsvp, dietary_requirements 
                FROM invitee WHERE id IN (SELECT unnest($1::TEXT[]))",
                &[&ids],
            )
            .await;

        if let Err(err) = result {
            event!(
                Level::ERROR,
                "Failed to run query for fetching multiple invitees"
            );
            return Err(RepoErr::DBFailure(err.to_string()));
        }
        let result = result.expect("Should handle err");

        let update_result = self
            .client
            .query(
                "UPDATE invitee SET invitation_opened = true WHERE id IN (SELECT unnest($1::TEXT[]))",
                &[&ids],
            )
            .await;

        if let Err(err) = update_result {
            event!(
                Level::ERROR,
                "Failed to update invitees' invitation_opened status"
            );
            return Err(RepoErr::DBFailure(err.to_string()));
        }

        let invitees: Result<Vec<InviteeDTO>, &str> =
            result.iter().map(|e| InviteeDTO::try_from(e)).collect();

        if let Err(err) = invitees {
            event!(Level::ERROR, "Failed to parse invitees from db result");
            return Err(RepoErr::DBFailure(err.to_string()));
        }

        Ok(invitees.unwrap())
    }

    #[tracing::instrument(skip(self))]
    async fn update_invitee(&self, invitee: &UpdateInviteeParams) -> Result<InviteeDTO, RepoErr> {
        let rsvp = match invitee.rsvp {
            Some(t) => {
                if t {
                    "Coming"
                } else {
                    "NotComing"
                }
            }
            None => "Unknown",
        };
        let result = self
            .client
            .query(
                "UPDATE invitee 
                SET rsvp = $1::TEXT, dietary_requirements = $2::text
                WHERE id = $3::TEXT
                RETURNING id, fname, lname, rsvp, dietary_requirements
            ",
                &[&rsvp, &invitee.dietary_requirements, &invitee.id],
            )
            .await;
        if let Err(err) = result {
            event!(Level::ERROR, "Failed to run query to rsvp");
            return Err(RepoErr::DBFailure(err.to_string()));
        }
        let result = result.expect("Should handle err");
        let result = result.get(0);
        if let None = result {
            event!(Level::ERROR, "Failed to find invitee");
            return Err(RepoErr::ItemNotFound(invitee.id.clone()));
        }
        let result = result.expect("Should handle None");

        let invitee = InviteeDTO::try_from(result).map_err(|e| RepoErr::DBFailure(e.to_string()));

        if let Err(err) = invitee {
            event!(
                Level::ERROR,
                msg = "Failed to parse db row into invitee",
                ?result
            );
            return Err(err);
        }

        Ok(invitee.expect("Should handle err"))
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn should_get_invitee_by_id() {
        let uri = env::var("DEV_POSTGRES_URI").expect("Uri should be defined for test");
        let (client, connection) =
            tokio_postgres::connect(&format!("{}/wedding_dev", uri), tokio_postgres::NoTls)
                .await
                .expect("Connection should not fail");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let id: String = Uuid::new_v4().to_string();

        // setup
        client
            .query(
                "
                INSERT INTO invitee (
                    id,
                    fname,
                    lname,
                    rsvp,
                    dietary_requirements,
                    invitation_opened
                ) VALUES (
                    $1::TEXT,
                    'Test1',
                    '1',
                    'UNKNOWN',
                    'something',
                    false
                );
                ",
                &[&id],
            )
            .await
            .expect("Insert query should not fail");

        // test
        let db = DB { client: &client };
        let invite = db.get_invitee_by_id(&id).await;
        let invite = invite.expect("Should retrieve sent");

        assert_eq!(invite.id, id);
        assert_eq!(invite.fname, "Test1".to_owned());
        assert_eq!(invite.lname, "1".to_owned());
        assert_eq!(invite.rsvp, None);
        assert_eq!(invite.dietary_requirements, "something".to_owned());

        //cleanup
        client
            .query("DELETE FROM invitee WHERE invitee.id = $1::TEXT", &[&id])
            .await
            .expect("Should delete created");
    }

    #[tokio::test]
    async fn should_get_invitee_by_ids() {
        let uri = env::var("DEV_POSTGRES_URI").expect("Uri should be defined for test");
        let (client, connection) =
            tokio_postgres::connect(&format!("{}/wedding_dev", uri), tokio_postgres::NoTls)
                .await
                .expect("Connection should not fail");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let id: String = Uuid::new_v4().to_string();
        let id2: String = Uuid::new_v4().to_string();
        let id3: String = Uuid::new_v4().to_string();

        // setup
        let _invite_1 = client
            .query(
                "
                INSERT INTO invitee (
                    id,
                    fname,
                    lname,
                    rsvp,
                    dietary_requirements,
                    invitation_opened
                ) VALUES (
                    $1::TEXT,
                    'Test1',
                    '1',
                    'UNKNOWN',
                    'something',
                    false
                );
                ",
                &[&id],
            )
            .await
            .expect("insert shouuld not fail");

        let _invite_2 = client
            .query(
                "
                INSERT INTO invitee (
                    id,
                    fname,
                    lname,
                    rsvp,
                    dietary_requirements,
                    invitation_opened
                ) VALUES (
                    $1::TEXT,
                    'Test1',
                    '1',
                    'UNKNOWN',
                    'something',
                    false
                );
                ",
                &[&id2],
            )
            .await
            .expect("insert shouuld not fail");

        let _invite_3 = client
            .query(
                "
                INSERT INTO invitee (
                    id,
                    fname,
                    lname,
                    rsvp,
                    dietary_requirements,
                    invitation_opened
                ) VALUES (
                    $1::TEXT,
                    'Test1',
                    '1',
                    'UNKNOWN',
                    'something',
                    false
                );
                ",
                &[&id3],
            )
            .await
            .expect("insert shouuld not fail");

        // test
        let db = DB { client: &client };
        let invites = db
            .get_invitee_by_ids(&vec![&id, &id2, &id3])
            .await
            .expect("Should retrieve invites");

        assert_eq!(invites.len(), 3);

        //cleanup
        client
            .query(
                "DELETE FROM invitee WHERE invitee.id IN (SELECT unnest($1::TEXT[]))",
                &[&vec![id, id2, id3]],
            )
            .await
            .expect("Should delete created");
    }

    #[tokio::test]
    async fn should_update_invitee() {
        let uri = env::var("DEV_POSTGRES_URI").expect("Uri should be defined for test");
        let (client, connection) =
            tokio_postgres::connect(&format!("{}/wedding_dev", uri), tokio_postgres::NoTls)
                .await
                .expect("Connection should not fail");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let id: String = Uuid::new_v4().to_string();

        // setup
        client
            .query(
                "
                INSERT INTO invitee (
                    id,
                    fname,
                    lname,
                    rsvp,
                    dietary_requirements,
                    invitation_opened
                ) VALUES (
                    $1::TEXT,
                    'Test1',
                    '1',
                    'UNKNOWN',
                    'something',
                    false
                );
                ",
                &[&id],
            )
            .await
            .expect("Insert query should not fail");

        // test
        let db = DB { client: &client };
        let params = UpdateInviteeParams {
            id: id.clone(),
            rsvp: Some(true),
            dietary_requirements: "Something new".to_string(),
        };
        let invite = db
            .update_invitee(&params)
            .await
            .expect("Should update invite");

        assert_eq!(invite.id, id);
        assert_eq!(invite.rsvp, Some(true));
        assert_eq!(invite.dietary_requirements, "Something new".to_string());

        //cleanup
        client
            .query("DELETE FROM invitee WHERE invitee.id = $1::TEXT", &[&id])
            .await
            .expect("Should delete created");
    }
}

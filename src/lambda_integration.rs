use super::*;

impl From<RepoErr> for HttpError {
    fn from(e: RepoErr) -> Self {
        match e {
            RepoErr::ItemNotFound(err) => Self {
                status_code: 400,
                err_type: "item-not-found".to_string(),
                msg: Some(err.to_string()),
            },
            RepoErr::DBFailure(err) => Self {
                status_code: 500,
                err_type: "db-failure".to_string(),
                msg: Some(err.to_string()),
            },
        }
    }
}

impl From<ApiErr> for HttpError {
    fn from(err: ApiErr) -> Self {
        match err {
            ApiErr::RepoErr(err) => err.into(),
            ApiErr::ArgumentErr(err) => Self {
                status_code: 400,
                err_type: "argument-err".to_string(),
                msg: Some(err.to_string()),
            },
        }
    }
}

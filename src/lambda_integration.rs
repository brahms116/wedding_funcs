use super::*;

impl From<ApiErr> for HttpError {
    fn from(err: ApiErr) -> Self {
        match err {
            ApiErr::RepoErr(err) => Self {
                status_code: 500,
                msg: Some(err.to_string()),
            },
            ApiErr::ArgumentErr(err) => Self {
                status_code: 400,
                msg: Some(err.to_string()),
            },
        }
    }
}

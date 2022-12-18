use super::*;

impl Into<HttpError> for ApiErr {
    fn into(self) -> HttpError {
        match self {
            Self::RepoErr(err) => HttpError {
                status_code: 500,
                msg: Some(err.to_string()),
            },
            Self::ArgumentErr(err) => HttpError {
                status_code: 400,
                msg: Some(err.to_string()),
            },
        }
    }
}

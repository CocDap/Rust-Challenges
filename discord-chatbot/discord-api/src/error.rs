
// Alias lại Result cho việc custom lỗi 
pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    Req(reqwest::Error),
    Json(serde_json::Error),
    Status(reqwest::StatusCode),
    Other(&'static str),
}

// Convert lỗi của thư viện reqwest sang lỗi custom của mình 
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Req(err)
    }
}
// Convert lỗi của thư viện serde_json sang lỗi custom của mình 
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}


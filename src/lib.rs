//! Interface to the imgur API.

#![warn(missing_docs)]

extern crate curl;
extern crate serde_json as json;
#[macro_use]
extern crate try_opt;

use json::Value;
use std::fmt;
use std::error::Error;

macro_rules! api_url (
    ($url: expr) => (
        concat!("https://api.imgur.com/3/", $url)
    );
);

/// A handle to the imgur API.
pub struct Handle {
    client_id: String,
}

impl Handle {
    /// Create a new handle.
    ///
    /// # Parameters
    ///
    /// client_id: Client ID required to access the imgur API.
    pub fn new(client_id: String) -> Self {
        Handle { client_id: client_id }
    }

    /// Upload image data to imgur.
    ///
    /// # Parameters
    ///
    /// data: The image data to upload.
    ///
    /// # Returns
    ///
    /// UploadInfo on success, UploadError on failure.
    pub fn upload(&self, data: &[u8]) -> Result<UploadInfo, UploadError> {
        use std::io::Cursor;
        let mut handle = curl::http::handle();
        let mut cursor = Cursor::new(data);
        let request = handle.post(api_url!("image"), &mut cursor)
                            .header("Authorization", &format!("Client-ID {}", self.client_id));
        let response = try!(request.exec());
        let text = try!(std::str::from_utf8(response.get_body()));
        Ok(UploadInfo {
            json: match json::from_str(text) {
                Ok(value) => value,
                Err(e) => {
                    let kind = UploadErrorKind::ResponseBodyInvalidJson(text.into(), e);
                    return Err(UploadError { kind: kind });
                }
            },
        })
    }
}

/// Information about an uploaded image.
pub struct UploadInfo {
    json: Value,
}

impl UploadInfo {
    /// Returns the link the image was uploaded to, if any.
    pub fn link(&self) -> Option<&str> {
        let data = try_opt!(self.json.find("data"));
        data.find("link").and_then(|v| v.as_string())
    }
}

#[derive(Debug)]
enum UploadErrorKind {
    CurlErrCode(curl::ErrCode),
    ResponseBodyInvalidUtf8(std::str::Utf8Error),
    ResponseBodyInvalidJson(String, json::Error),
}

#[derive(Debug)]
/// Error that can happen on image upload.
pub struct UploadError {
    kind: UploadErrorKind,
}

impl From<curl::ErrCode> for UploadError {
    fn from(src: curl::ErrCode) -> Self {
        UploadError { kind: UploadErrorKind::CurlErrCode(src) }
    }
}

impl From<std::str::Utf8Error> for UploadError {
    fn from(src: std::str::Utf8Error) -> Self {
        UploadError { kind: UploadErrorKind::ResponseBodyInvalidUtf8(src) }
    }
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use UploadErrorKind::*;
        match self.kind {
            CurlErrCode(code) => write!(f, "Curl error code: {}", code),
            ResponseBodyInvalidUtf8(err) => {
                write!(f, "Response body is not valid utf-8: {}", err)
            }
            ResponseBodyInvalidJson(ref body, ref err) => {
                write!(f,
                       "Response body is not valid json. body: {:?}, err: {}",
                       body,
                       err)
            }
        }
    }
}

impl Error for UploadError {
    fn description(&self) -> &str {
        "Image upload error"
    }
}

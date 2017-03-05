//! Interface to the imgur API.

#![warn(missing_docs)]

extern crate hyper;
extern crate hyper_rustls;
extern crate serde_json as json;

use json::Value;
use std::fmt;
use std::error::Error;
use std::io;

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
        use hyper::Client;
        use hyper::header::Authorization;
        use hyper::net::HttpsConnector;
        use std::io::Read;

        let client = Client::with_connector(HttpsConnector::new(hyper_rustls::TlsClient::new()));
        let mut response = client.post(api_url!("image"))
            .header(Authorization(format!("Client-ID {}", self.client_id)))
            .body(data)
            .send()?;
        let mut text = String::new();
        response.read_to_string(&mut text)?;
        Ok(UploadInfo {
            json: match json::from_str(&text) {
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
        self.json.get("data").and_then(|data| data.get("link").and_then(|v| v.as_str()))
    }
}

#[derive(Debug)]
enum UploadErrorKind {
    Hyper(hyper::Error),
    Io(io::Error),
    ResponseBodyInvalidUtf8(std::str::Utf8Error),
    ResponseBodyInvalidJson(String, json::Error),
}

#[derive(Debug)]
/// Error that can happen on image upload.
pub struct UploadError {
    kind: UploadErrorKind,
}

impl From<std::str::Utf8Error> for UploadError {
    fn from(src: std::str::Utf8Error) -> Self {
        UploadError { kind: UploadErrorKind::ResponseBodyInvalidUtf8(src) }
    }
}

impl From<hyper::Error> for UploadError {
    fn from(src: hyper::Error) -> Self {
        Self { kind: UploadErrorKind::Hyper(src) }
    }
}

impl From<io::Error> for UploadError {
    fn from(src: io::Error) -> Self {
        Self { kind: UploadErrorKind::Io(src) }
    }
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use UploadErrorKind::*;
        match self.kind {
            Hyper(ref err) => write!(f, "Hyper error: {}", err),
            Io(ref err) => write!(f, "I/O error: {}", err),
            ResponseBodyInvalidUtf8(err) => write!(f, "Response body is not valid utf-8: {}", err),
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

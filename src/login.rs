use std::fmt::{Debug, Display, Formatter};
use crate::LoginError::RequestError;
use crate::login::LoginError::{BadRequest, BadStatusCode};

#[derive(serde::Serialize)]
struct LoginRequest {
    client_id: &'static str,
    client_secret: &'static str,
    redirect_uri: &'static str,
    grant_type: &'static str,
    username: String,
    password: String,
    scope: &'static str
}

#[derive(serde::Deserialize)]
struct LoginFailureResponse {
    /// The error message explaining the reason for the error.
    error: String
}

/// A login-related error.
#[derive(Debug)]
pub(crate) enum LoginError {
    /// The request was fulfilled, but the server responded with `HTTP 400 Bad Request`.
    BadRequest {
        /// The server-provided error message.
        error: String
    },

    /// The request was fulfilled, but the status code is neither 2xx or 4xx.
    BadStatusCode {
        status_code: reqwest::StatusCode
    },

    /// The request was not executed because an error occurred.
    RequestError {
        error: reqwest::Error
    }
}

impl Display for LoginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BadRequest { error } => write!(f, "bad request: {}", error),
            BadStatusCode { status_code } => write!(f, "bad status code: {}", status_code),
            RequestError { error } => write!(f, "request error: {}", error)
        }
    }
}

impl std::error::Error for LoginError {}

impl From<reqwest::Error> for LoginError {
    fn from(error: reqwest::Error) -> Self {
        RequestError {
            error
        }
    }
}

/// Attempts to login with the given client, username and password.
///
/// The returned bool is `true` if the given credentials are valid.
pub(crate) async fn login(client: &reqwest::Client, username: String, password: String) -> Result<bool, LoginError> {
    // Send login request
    let response = client
        .post("https://meatbag.app/oauth/token")
        .json(&LoginRequest{
            client_id: "",
            client_secret: "",
            redirect_uri: "urn:ietf:wg:oauth:2.0:oob",
            grant_type: "password",
            username,
            password,
            scope: "read write follow push admin"
        })
        .send()
        .await?;

    let status = response.status();

    return if status.is_success() {
        // Assume success if status is 2xx
        Ok(true)
    } else if status.is_client_error() {
        // Assume error if status is 4xx
        let body = response.json::<LoginFailureResponse>().await?;

        if body.error == "Invalid credentials" {
            // Credentials are invalid
            Ok(false)
        } else {
            // Bad request
            Err(BadRequest {
                error: body.error
            })
        }
    } else {
        // Otherwise, this is an unknown error
        Err(BadStatusCode {
            status_code: status
        })
    }
}

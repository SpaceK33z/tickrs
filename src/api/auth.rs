//! OAuth 2.0 authentication flow for TickTick API
//!
//! Implements the OAuth authorization code flow:
//! 1. Generate authorization URL
//! 2. Open browser for user to authorize
//! 3. Capture callback with authorization code
//! 4. Exchange code for access token

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use anyhow::{anyhow, Context, Result};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::redirect::Policy;

use crate::constants::{OAUTH_AUTH_URL, OAUTH_REDIRECT_URI, OAUTH_SCOPES, OAUTH_TOKEN_URL};

/// OAuth authentication handler
pub struct AuthHandler {
    client_id: String,
    client_secret: String,
}

impl AuthHandler {
    /// Create a new auth handler with client credentials
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    /// Generate the authorization URL for the user to visit
    pub fn get_auth_url(&self) -> Result<(String, CsrfToken)> {
        let client = self.create_oauth_client()?;

        let mut auth_request = client.authorize_url(CsrfToken::new_random);

        // Add scopes
        for scope in OAUTH_SCOPES {
            auth_request = auth_request.add_scope(Scope::new((*scope).to_string()));
        }

        let (auth_url, csrf_token) = auth_request.url();
        Ok((auth_url.to_string(), csrf_token))
    }

    /// Run the full OAuth flow: open browser, capture callback, exchange code
    pub async fn run_oauth_flow(&self) -> Result<String> {
        let (auth_url, csrf_token) = self.get_auth_url()?;

        // Try to open browser, but don't fail if it can't open (e.g., headless environments)
        let _ = webbrowser::open(&auth_url);

        // Wait for callback with authorization code
        let code = self.capture_callback(csrf_token)?;

        // Exchange code for token
        let token = self.exchange_code(&code).await?;

        Ok(token)
    }

    /// Capture the OAuth callback on localhost
    fn capture_callback(&self, expected_csrf: CsrfToken) -> Result<String> {
        // Bind to localhost:8080
        let listener = TcpListener::bind("127.0.0.1:8080")
            .context("Failed to bind to localhost:8080. Is another process using this port?")?;

        // Accept a single connection
        let (mut stream, _) = listener
            .accept()
            .context("Failed to accept OAuth callback connection")?;

        // Read the request
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .context("Failed to read OAuth callback request")?;

        // Parse the request to extract code and state
        let (code, state) = parse_callback_request(&request_line)?;

        // Verify CSRF token
        if state != *expected_csrf.secret() {
            // Send error response
            let response = create_error_response("CSRF token mismatch - possible security issue");
            stream.write_all(response.as_bytes())?;
            return Err(anyhow!(
                "CSRF token mismatch - authorization may have been tampered with"
            ));
        }

        // Send success response
        let response = create_success_response();
        stream.write_all(response.as_bytes())?;

        Ok(code)
    }

    /// Exchange authorization code for access token
    async fn exchange_code(&self, code: &str) -> Result<String> {
        let client = self.create_oauth_client()?;

        // Create HTTP client with no redirects for SSRF protection
        let http_client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .context("Failed to create HTTP client")?;

        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&http_client)
            .await
            .context("Failed to exchange authorization code for token")?;

        Ok(token_result.access_token().secret().clone())
    }

    /// Create the OAuth2 client with auth and token URLs configured
    #[allow(clippy::type_complexity)]
    fn create_oauth_client(
        &self,
    ) -> Result<
        oauth2::Client<
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
            oauth2::StandardTokenResponse<
                oauth2::EmptyExtraTokenFields,
                oauth2::basic::BasicTokenType,
            >,
            oauth2::StandardTokenIntrospectionResponse<
                oauth2::EmptyExtraTokenFields,
                oauth2::basic::BasicTokenType,
            >,
            oauth2::StandardRevocableToken,
            oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
            oauth2::EndpointSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointSet,
        >,
    > {
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new(OAUTH_AUTH_URL.to_string()).context("Invalid authorization URL")?,
            )
            .set_token_uri(TokenUrl::new(OAUTH_TOKEN_URL.to_string()).context("Invalid token URL")?)
            .set_redirect_uri(
                RedirectUrl::new(OAUTH_REDIRECT_URI.to_string()).context("Invalid redirect URI")?,
            );

        Ok(client)
    }
}

/// Parse the OAuth callback request to extract code and state
fn parse_callback_request(request_line: &str) -> Result<(String, String)> {
    // Request line format: "GET /?code=xxx&state=yyy HTTP/1.1"
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow!("Invalid callback request format"));
    }

    let path = parts[1];

    // Check for error in callback
    if path.contains("error=") {
        let error_desc = extract_param(path, "error_description")
            .or_else(|| extract_param(path, "error"))
            .unwrap_or_else(|| "Unknown authorization error".to_string());
        return Err(anyhow!("Authorization failed: {}", error_desc));
    }

    let code =
        extract_param(path, "code").ok_or_else(|| anyhow!("No authorization code in callback"))?;

    let state =
        extract_param(path, "state").ok_or_else(|| anyhow!("No state parameter in callback"))?;

    Ok((code, state))
}

/// Extract a query parameter value from a path
fn extract_param(path: &str, param: &str) -> Option<String> {
    let query = path.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut kv = pair.split('=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            if key == param {
                return Some(urlencoding_decode(value));
            }
        }
    }
    None
}

/// Simple URL decoding (handles common cases)
fn urlencoding_decode(s: &str) -> String {
    s.replace("%20", " ")
        .replace("%21", "!")
        .replace("%2B", "+")
        .replace("%3D", "=")
        .replace("%26", "&")
        .replace("%3F", "?")
        .replace("%2F", "/")
        .replace("%3A", ":")
}

/// Create an HTTP success response
fn create_success_response() -> String {
    let body = r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorization Successful</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
               display: flex; justify-content: center; align-items: center; height: 100vh;
               margin: 0; background: #f5f5f5; }
        .container { text-align: center; padding: 40px; background: white;
                     border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h1 { color: #22c55e; margin-bottom: 16px; }
        p { color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Authorization Successful</h1>
        <p>You can close this window and return to the terminal.</p>
    </div>
</body>
</html>"#;

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

/// Create an HTTP error response
fn create_error_response(message: &str) -> String {
    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorization Failed</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
               display: flex; justify-content: center; align-items: center; height: 100vh;
               margin: 0; background: #f5f5f5; }}
        .container {{ text-align: center; padding: 40px; background: white;
                     border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #ef4444; margin-bottom: 16px; }}
        p {{ color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Authorization Failed</h1>
        <p>{}</p>
    </div>
</body>
</html>"#,
        message
    );

    format!(
        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callback_request_success() {
        let request = "GET /?code=abc123&state=xyz789 HTTP/1.1";
        let (code, state) = parse_callback_request(request).unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn test_parse_callback_request_error() {
        let request = "GET /?error=access_denied&error_description=User%20denied HTTP/1.1";
        let result = parse_callback_request(request);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Authorization failed"));
    }

    #[test]
    fn test_parse_callback_request_missing_code() {
        let request = "GET /?state=xyz789 HTTP/1.1";
        let result = parse_callback_request(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_callback_request_missing_state() {
        let request = "GET /?code=abc123 HTTP/1.1";
        let result = parse_callback_request(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_param() {
        let path = "/?code=abc&state=xyz&other=123";
        assert_eq!(extract_param(path, "code"), Some("abc".to_string()));
        assert_eq!(extract_param(path, "state"), Some("xyz".to_string()));
        assert_eq!(extract_param(path, "other"), Some("123".to_string()));
        assert_eq!(extract_param(path, "missing"), None);
    }

    #[test]
    fn test_urlencoding_decode() {
        assert_eq!(urlencoding_decode("hello%20world"), "hello world");
        assert_eq!(urlencoding_decode("test%3Dvalue"), "test=value");
    }

    #[test]
    fn test_create_success_response() {
        let response = create_success_response();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Authorization Successful"));
    }

    #[test]
    fn test_create_error_response() {
        let response = create_error_response("Test error");
        assert!(response.starts_with("HTTP/1.1 400 Bad Request"));
        assert!(response.contains("Authorization Failed"));
        assert!(response.contains("Test error"));
    }
}

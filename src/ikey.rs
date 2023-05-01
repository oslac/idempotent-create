use axum::http::StatusCode;
use hyper::HeaderMap;
use std::fmt::Display;

/// An idempotency key is any identifier provided by a client in the header
/// `Idempotency-Key`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IKey(pub String);

impl IKey {
    /// The expected header (*key*).
    pub const HEADER: &str = "Idempotency-Key";
    /// The maximum length for the *value*.
    pub const MAX_LEN: u8 = 255;

    /// Parse and validate [IKey] from request headers.
    pub fn from_headers(headers: &HeaderMap) -> Result<IKey, (StatusCode, String)> {
        let Some(value) = headers
            .get(Self::HEADER)
            .and_then(|value| value.to_str().ok()) else {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "IKey: Heder received, value missing".to_string()))
            };

        let ikey = IKey::try_from(value.to_string())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        Ok(ikey)
    }
}

impl TryFrom<String> for IKey {
    type Error = String;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        if key.is_empty() {
            return Err("Empty Idempotency Key".to_string());
        };
        if key.len() >= IKey::MAX_LEN as usize {
            return Err(format!(
                "Idempotency Key Max Length is {}, but got {}",
                IKey::MAX_LEN,
                key.len()
            ));
        }

        Ok(Self(key))
    }
}

impl From<IKey> for String {
    fn from(key: IKey) -> Self {
        key.0
    }
}

impl AsRef<str> for IKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for IKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

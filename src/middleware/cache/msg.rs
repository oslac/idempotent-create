use crate::ikey::IKey;
use crate::warehouse::CacheError;
use crate::warehouse::CachedResponse;

use std::fmt::Display;
use tokio::sync::oneshot;

/// - Responder is provided by the **client** of *manager*, iow. the *request*.
/// - Responder is used by the **manager** to send the response back to the
///  requester.
type Responder<T> = oneshot::Sender<T>;
type GetResponder = Responder<Option<CachedResponse>>;
type SetResponder = Responder<Result<(), CacheError>>;

/// Defines the message types [CacheManager] and [CacheHandle] support.
#[derive(Debug)]
pub enum Msg {
    Get { key: IKey, ret: GetResponder },
    Set { key: IKey, val: CachedResponse, ret: SetResponder },
}

impl Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Msg::Get { key, ret: _ } => write!(f, "GET with (k: {key})"),
            Msg::Set { key, val, ret: _ } => write!(f, "SET with (k: {key}, v: {})", val.user.id),
        }
    }
}

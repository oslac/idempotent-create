use crate::ikey::IKey;
use crate::warehouse::CacheError;
use crate::warehouse::CachedResponse;
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

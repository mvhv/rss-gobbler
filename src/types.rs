use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

pub type BoxedSendSyncError = Box<dyn std::error::Error + Send + Sync>;
pub type AsyncResult<T> = Result<T, BoxedSendSyncError>;
pub type HttpsClient = hyper::Client<HttpsConnector<HttpConnector>>;
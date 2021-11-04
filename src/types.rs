pub type BoxedSendSyncError = Box<dyn std::error::Error + Send + Sync>;
pub type AsyncResult<T> = Result<T, BoxedSendSyncError>;

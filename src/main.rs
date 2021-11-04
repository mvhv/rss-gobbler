mod app;
mod config;
mod types;

use crate::app::run;
use crate::config::AppConfig;
use crate::types::AsyncResult;

#[tokio::main]
async fn main() -> AsyncResult<()> {
    // install global tracer
    tracing_subscriber::fmt::init();
    // init global config and run app
    run(AppConfig::from_cli_args()?).await
}

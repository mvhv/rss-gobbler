mod config;
mod types;
mod app;

use crate::types::AsyncResult;
use crate::config::AppConfig;
use crate::app::run;


#[tokio::main]
async fn main() -> AsyncResult<()> {
    // install global tracer
    tracing_subscriber::fmt::init();
    // init global config and run app
    run(AppConfig::from_cli_args()?).await
}
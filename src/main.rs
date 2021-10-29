mod config;

use config::AppConfig;

use hyper::{
    body::{self, HttpBody},
    client::HttpConnector,
    Client, Uri,
};

use hyper_tls::HttpsConnector;

use rss::{Channel, Enclosure, Item};

use tokio::{fs, io::AsyncWriteExt as _, task};

use futures::stream::{FuturesUnordered, StreamExt};

use tracing::{error, info};

use std::sync::Arc;

pub type BoxedSendSyncError = Box<dyn std::error::Error + Send + Sync>;
pub type AsyncResult<T> = Result<T, BoxedSendSyncError>;
pub type HttpsClient = hyper::Client<HttpsConnector<HttpConnector>>;


#[tokio::main]
async fn main() -> AsyncResult<()> {
    // install global tracer
    tracing_subscriber::fmt::init();

    let config = Arc::new(AppConfig::from_cli_args()?);
 

    // setup https client
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    // grab rss channel
    let channel = get_rss_channel(client.clone(), Arc::clone(&config)).await?;

    // spawn new download task for each item and collect futures
    let mut downloads = channel
        .items()
        .iter()
        .map(|item| {
            let (item, client, config) = (item.clone(), client.clone(), Arc::clone(&config));
            task::spawn(async move { download_enclosure(item, client, config).await })
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(handle) = downloads.next().await {
        if let Err(error) = handle? {
            error!("Error: {}", error);
        }
    }

    Ok(())
}

fn filename_from_title(title: &str) -> String {
    let filename = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();

    filename + ".mp3"
}

async fn create_file_in_dir(filename: &str, directory: &str) -> AsyncResult<fs::File> {
    fs::DirBuilder::new()
        .recursive(true)
        .create(directory)
        .await?;

    let io_result = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{}/{}", directory, filename))
        .await;

    match io_result {
        Ok(file) => Ok(file),
        Err(error) => Err(error.into()),
    }
}

async fn get_redirect_until(
    url: Uri,
    client: HttpsClient,
    max_hops: u8,
) -> AsyncResult<hyper::Response<hyper::Body>> {
    let mut location = url.clone();
    for _ in 0..max_hops {
        let resp = client.get(location.clone()).await?;
        match u16::from(resp.status()) {
            200 => {
                return Ok(resp);
            }
            code @ 300..=310 => {
                let prev = location.clone();
                location = resp
                    .headers()
                    .get(hyper::header::LOCATION)
                    .ok_or_else::<BoxedSendSyncError, _>(|| {
                        format!(
                            "HTTP: {} Redirect LOCATION field missing for GET: {}",
                            code, &location
                        )
                        .into()
                    })?
                    .to_str()
                    .map_err::<BoxedSendSyncError, _>(|_| {
                        format!(
                            "HTTP: {} Redirect LOCATION field is not a string for GET: {}",
                            code, &location
                        )
                        .into()
                    })?
                    .parse()?;
                info!("HTTP: {} Redirecting: {} -> {}", code, &prev, &location);
                continue;
            }
            code => {
                return Err(format!(
                    "HTTP: {} Unhandled status code for GET: {}",
                    code, &location
                )
                .into());
            }
        };
    }

    Err(format!(
        "HTTP exceeded max redirects: {} final host: {} for GET: {}",
        max_hops, &location, &url
    )
    .into())
}

async fn download_audio_file(url: Uri, title: &str, client: HttpsClient, config: Arc<AppConfig>) -> AsyncResult<()> {
    let filename = filename_from_title(title);
    let mut file = create_file_in_dir(&filename, &config.get_output_directory()).await?;

    info!("Downloading file: {}", &filename);
    let mut resp = get_redirect_until(url, client, 10).await?;
    while let Some(chunk) = resp.body_mut().data().await {
        file.write_all(&chunk?).await?;
    }
    info!("Download complete: {}", &filename);

    Ok(())
}

async fn download_enclosure(item: Item, client: HttpsClient, config: Arc<AppConfig>) -> AsyncResult<()> {
    if let Item {
        title: Some(title),
        enclosure: Some(Enclosure { url, .. }),
        ..
    } = item
    {
        info!("Parsed RSS item: {} with enclosure at: {}", &title, &url);
        if config.check_valid(&title) {
            download_audio_file(url.parse()?, &title, client, config).await
        } else {
            info!("Skipping due to regex rules: {}", &title);
            Ok(())
        }
    } else {
        Err(format!("Failed to parse RSS item: {:?}", item).into())
    }
}

async fn get_rss_channel(client: HttpsClient, config: Arc<AppConfig>) -> AsyncResult<Channel> {
    let resp = client.get(config.get_feed_uri()).await?;
    let content = body::to_bytes(resp.into_body()).await?;
    let channel = Channel::read_from(&content[..])?;
    info!(
        "Got RSS channel: Title: {} URL: {} Description: {}",
        channel.title(),
        channel.link(),
        channel.description()
    );

    Ok(channel)
}

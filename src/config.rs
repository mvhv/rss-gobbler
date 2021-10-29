use crate::AsyncResult;

use clap::{App, Arg};

use regex::{self, Regex};

use hyper::Uri;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const DEFAULT_DIRECTORY: &str = "episodes";

#[derive(Debug)]
pub struct AppConfig {
    feed_uri: Uri,
    output_path: String,
    option_include_regex: Option<Regex>,
    option_exclude_regex: Option<Regex>,
}


impl AppConfig {
    pub fn new(feed_url: &str, output_path: &str, include_pattern: Option<&str>, exclude_pattern: Option<&str>) -> Result<AppConfig, regex::Error> {
        let option_include_regex = match include_pattern {
            Some(pattern) => Some(Regex::new(pattern.into())?),
            None => None,
        };

        let option_exclude_regex = match exclude_pattern {
            Some(pattern) => Some(Regex::new(pattern.into())?),
            None => None,
        };

        let feed_uri = feed_url.parse().unwrap();
        
        Ok(AppConfig{ feed_uri, output_path: String::from(output_path), option_include_regex, option_exclude_regex})
    }

    pub fn get_feed_uri(&self) -> Uri {
        self.feed_uri.clone()
    }

    pub fn get_output_directory(&self) -> String {
        self.output_path.clone()
    }

    // fn include(&self) -> Option<&Regex> {
    //     match self.option_include_regex {
    //         Some(r) => &r,
    //         _ => None,
    //     }
    // }

    // fn exclude(&self) -> Option<&Regex> {
    //     match self.option_exclude_regex {
    //         Some(r) => &r,
    //         _ => None,
    //     }
    // }

    pub fn check_valid(&self, pattern: &str) -> bool {
        let include = if let Some(regex) = &self.option_include_regex {
            regex.is_match(pattern)
        } else {
            true
        };

        let exclude = if let Some(regex) = &self.option_exclude_regex {
            regex.is_match(pattern)
        } else {
            false
        };

        include && !exclude
    }

    
    pub fn from_cli_args() -> AsyncResult<AppConfig> {
        // parse cmdline args
        let matches = App::new("RSS Gobbler")
            .version(APP_VERSION)
            .author(APP_AUTHORS)
            .about(APP_DESCRIPTION)
            .arg(
                Arg::with_name("feed_url")
                    .short("f")
                    .long("feed")
                    .value_name("URL")
                    .help("The URL of the RSS feed to download.")
                    .required(true)
            )
            .arg(
                Arg::with_name("directory")
                    .short("d")
                    .long("dir")
                    .value_name("OUTPUT_PATH")
                    .help("The path to store downloaded episodes.")
                    .default_value(DEFAULT_DIRECTORY)
            )
            .arg(
                Arg::with_name("include_pattern")
                    .short("i")
                    .long("include")
                    .value_name("REGEX_PATTERN")
                    .help("An optional regex pattern to for episodes to include.")
                    .required(false)
            )
            .arg(
                Arg::with_name("exclude_pattern")
                    .short("e")
                    .long("exclude")
                    .value_name("REGEX_PATTERN")
                    .help("An optional regex pattern to for episodes to exclude.")
                    .required(false)
            )
            .get_matches();
        
        let feed_url = matches.value_of("feed_url").unwrap();
        let output_path = matches.value_of("feed_url").unwrap();
        let include_pattern = matches.value_of("include_pattern");
        let exclude_pattern = matches.value_of("exclude_pattern");

        match AppConfig::new(feed_url, output_path, include_pattern, exclude_pattern) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!("Failed to build AppConfig: {}", err).into()),
        }
    }
}


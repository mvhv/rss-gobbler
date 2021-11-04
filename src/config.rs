use crate::AsyncResult;

use clap::{App, Arg};
use hyper::Uri;
use regex::Regex;

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
    /// Constructs a new AppConfig including compiled regular expressions given input patterns
    pub fn new(
        feed_url: &str,
        output_path: &str,
        include_pattern: Option<&str>,
        exclude_pattern: Option<&str>,
    ) -> Result<AppConfig, regex::Error> {
        let option_include_regex = match include_pattern {
            Some(pattern) => Some(Regex::new(pattern)?),
            None => None,
        };

        let option_exclude_regex = match exclude_pattern {
            Some(pattern) => Some(Regex::new(pattern)?),
            None => None,
        };

        let feed_uri = feed_url.parse().unwrap();

        Ok(AppConfig {
            feed_uri,
            output_path: String::from(output_path),
            option_include_regex,
            option_exclude_regex,
        })
    }

    pub fn get_feed_uri(&self) -> Uri {
        self.feed_uri.clone()
    }

    pub fn get_output_directory(&self) -> String {
        self.output_path.clone()
    }

    pub fn is_pattern_valid(&self, pattern: &str) -> bool {
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

    /// Parse clap commandline arguments, and construct a new AppConfig
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
                    .required(true),
            )
            .arg(
                Arg::with_name("directory")
                    .short("d")
                    .long("dir")
                    .value_name("OUTPUT_PATH")
                    .help("The path to store downloaded episodes.")
                    .default_value(DEFAULT_DIRECTORY),
            )
            .arg(
                Arg::with_name("include_pattern")
                    .short("i")
                    .long("include")
                    .value_name("REGEX_PATTERN")
                    .help("An optional regex pattern to for episodes to include.")
                    .required(false),
            )
            .arg(
                Arg::with_name("exclude_pattern")
                    .short("e")
                    .long("exclude")
                    .value_name("REGEX_PATTERN")
                    .help("An optional regex pattern to for episodes to exclude.")
                    .required(false),
            )
            .get_matches();
        let feed_url = matches.value_of("feed_url").unwrap();
        let output_path = matches.value_of("feed_url").unwrap();
        let include_pattern = matches.value_of("include_pattern");
        let exclude_pattern = matches.value_of("exclude_pattern");
        // compile regex and return config
        match AppConfig::new(feed_url, output_path, include_pattern, exclude_pattern) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!("Failed to build AppConfig: {}", err).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_appconfig(has_some_regex: bool) -> AppConfig {
        let feed_url = "https://rss.example.com/podcast";
        let output_path = "/output/path";
        let include_pattern = r"^dog";
        let exclude_pattern = r"c.*t";

        AppConfig::new(
            feed_url,
            output_path,
            if has_some_regex {
                Some(include_pattern)
            } else {
                None
            },
            if has_some_regex {
                Some(exclude_pattern)
            } else {
                None
            },
        )
        .unwrap()
    }

    #[test]
    fn test_appconfig_regex() {
        let config_none = mock_appconfig(false);
        assert_eq!(config_none.is_pattern_valid("dog episode"), true);
        assert_eq!(config_none.is_pattern_valid("episode"), true);
        assert_eq!(config_none.is_pattern_valid("dog casdat"), true);

        let config_some = mock_appconfig(true);
        assert_eq!(config_some.is_pattern_valid("dog episode"), true); // meets start with dog
        assert_eq!(config_some.is_pattern_valid("episode"), false); // doesn't start with dog
        assert_eq!(config_some.is_pattern_valid("dog casdat"), false); // starts with dog but contains c.*t
    }
}

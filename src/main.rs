use arboard::Clipboard;
use memoize::memoize;
use miette::{miette, Result};
use std::{env, time::Duration};
use url::Url;
use wildmatch::WildMatch;

mod config;
use config::Config;

/// How often should clipboard be checked for changes (0 will result in high CPU usage)
const ITERATION_DELAY: Duration = Duration::from_millis(250);

fn main() -> Result<()> {
    // Get filter file path
    let path = env::args()
        .nth(1)
        .ok_or_else(|| miette!("Please provide a path to a KDL file with parameter filters"))?;

    let filters = Config::from_file(&path)?;

    println!("Loaded with categories:");
    for filter in &*filters {
        println!("\t• {}", filter.name);
    }

    // Initialize clipboard context
    let mut clipboard = Clipboard::new()
        .map_err(|e| miette!(format!("Could not initialize clipboard context: {e}")))?;
    let mut last_contents = clipboard.get_text().unwrap_or_else(|_| String::new());
    loop {
        std::thread::sleep(ITERATION_DELAY);
        if let Ok(contents) = clipboard.get_text() {
            // Empty clipboard (Linux)
            if contents.is_empty() {
                continue;
            };

            // Clipboard changed
            if contents != last_contents {
                last_contents = contents.clone();
                if let Ok(url) = clean_url(contents, filters.flat()) {
                    // Update clipboard
                    clipboard
                        .set_text(url.clone())
                        .map_err(|e| miette!(format!("Couldn't set clipboard contents: {e}")))?;
                    last_contents = url;
                };
            };
        }
    }
}

#[memoize(Capacity: 1024)]
fn clean_url(text: String, patterns: Vec<String>) -> Result<String, String> {
    let mut url = Url::parse(&text).map_err(|e| format!("Contents are not a valid URL: {e}"))?;
    let url_inner = url.clone();

    // Skip URLs without a host
    let Some(host) = url_inner.host_str() else {
        return Err(format!("URL {url_inner} does not have a host"));
    };

    // Handle URLs without query parameters
    if url.query().is_none() {
        return Ok(url.to_string());
    }

    for pattern in &patterns {
        let url_inner = url.clone();
        if let Some((param, domain)) = pattern.split_once('@') {
            if WildMatch::new(domain).matches(host) {
                // Filter parameters to exclude blocked entries
                let query = url_inner
                    .query_pairs()
                    .filter(|x| !WildMatch::new(param).matches(&x.0));
                // Replace parameters in URL
                url.query_pairs_mut().clear().extend_pairs(query);
            }
        } else {
            // Filter parameters to exclude blocked entries
            let query = url_inner
                .query_pairs()
                .filter(|x| !WildMatch::new(pattern).matches(&x.0));
            // Replace parameters in URL
            url.query_pairs_mut().clear().extend_pairs(query);
        }
    }

    Ok(url.to_string())
}

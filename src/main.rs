use cli_clipboard::{ClipboardContext, ClipboardProvider};
use memoize::memoize;
use miette::{miette, IntoDiagnostic, Result};
use std::{env, fs};
use url::Url;
use wildmatch::WildMatch;

#[derive(knuffel::Decode, Debug)]
struct Category {
    #[knuffel(argument)]
    name: String,
    #[knuffel(property, default)]
    disabled: bool,
    #[knuffel(child, unwrap(arguments))]
    params: Vec<String>,
}

fn main() -> Result<()> {
    // Get filter file path
    let filters = env::args()
        .nth(1)
        .ok_or_else(|| miette!("Please provide a path to a KDL file with parameter filters"))?;

    // Read filter file
    let filters = fs::read_to_string(&filters)
        .into_diagnostic()
        .map_err(|err| err.context(format!("Could not read file `{filters}`")))?;

    let filters = knuffel::parse::<Vec<Category>>("config.kdl", &filters)?
        .into_iter()
        .filter(|v| !v.disabled)
        .collect::<Vec<Category>>();

    println!("Loaded with categories:");
    filters.iter().for_each(|v| println!("\t• {}", v.name));

    // Flatten filters into patterns
    let patterns: Vec<String> = filters.iter().map(|v| v.params.clone()).flatten().collect();

    // Initialize clipboard context
    let mut clipboard = ClipboardContext::new()
        .map_err(|e| miette!(format!("Could not initialize clipboard context: {e}")))?;
    let mut last_contents = clipboard.get_contents().unwrap_or_else(|_| String::new());
    loop {
        match clipboard.get_contents() {
            Ok(contents) => {
                // Empty clipboard (Linux)
                if contents.is_empty() {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    continue;
                };

                // Clipboard changed
                if contents != last_contents {
                    last_contents = contents.clone();
                    if let Ok(url) = clean_url(contents, patterns.clone()) {
                        // Update clipboard
                        clipboard.set_contents(url.clone()).map_err(|e| {
                            miette!(format!("Couldn't set clipboard contents: {e}"))
                        })?;
                        last_contents = url;
                    };
                };
            }
            // Empty clipboard (Mac, Windows)
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(250));
                continue;
            }
        };
    }
}

#[memoize(Capacity: 1024)]
fn clean_url(text: String, patterns: Vec<String>) -> Result<String, String> {
    if let Ok(mut url) = Url::parse(&text) {
        let url_inner = url.clone();

        // Skip URLs without a host
        let Some(host) = url_inner.host_str() else { return Err(format!("URL {} does not have a host", url_inner)) };

        for pattern in &patterns {
            let url_inner = url.clone();
            match pattern.split_once('@') {
                Some((param, domain)) => {
                    if WildMatch::new(domain).matches(host) {
                        // Filter parameters to exclude blocked entries
                        let query = url_inner
                            .query_pairs()
                            .filter(|x| !WildMatch::new(param).matches(&x.0));
                        // Replace parameters in URL
                        url.query_pairs_mut().clear().extend_pairs(query);
                    }
                }
                None => {
                    // Filter parameters to exclude blocked entries
                    let query = url_inner
                        .query_pairs()
                        .filter(|x| !WildMatch::new(pattern).matches(&x.0));
                    // Replace parameters in URL
                    url.query_pairs_mut().clear().extend_pairs(query);
                }
            }
        }

        // Handle dangling ?s when no query pairs are appended
        let url = url.as_str().trim_end_matches("?").to_owned();

        Ok(url)
    } else {
        Err(format!("Contents are not a valid URL"))
    }
}

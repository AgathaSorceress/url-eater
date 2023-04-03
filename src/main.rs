use cli_clipboard::{ClipboardContext, ClipboardProvider};
use miette::{miette, IntoDiagnostic, Result};
use std::{env, fs};
use url::Url;
use wildmatch::WildMatch;

#[derive(knuffel::Decode, Debug)]
struct Category {
    #[knuffel(argument)]
    name: String,
    #[knuffel(child, unwrap(arguments))]
    params: Vec<String>,
}

fn main() -> Result<()> {
    // Get configuration file path
    let config = env::args()
        .nth(1)
        .ok_or_else(|| miette!("Please provide a path to a KDL file with blocked parameters"))?;

    // Read configurtaion file
    let config = fs::read_to_string(&config)
        .into_diagnostic()
        .map_err(|err| err.context(format!("Could not read file `{config}`")))?;

    let config = knuffel::parse::<Vec<Category>>("config.kdl", &config)?;

    println!("Loaded with categories:");
    config.iter().for_each(|v| println!("\t• {}", v.name));

    // Flatten all patterns into a single list, as categories do not matter
    let patterns: Vec<String> = config.iter().map(|v| v.params.clone()).flatten().collect();

    // Initialize clipboard context
    let mut clipboard = ClipboardContext::new()
        .map_err(|e| miette!(format!("Could not initialize clipboard context: {e}")))?;
    let mut last_contents = clipboard.get_contents().unwrap_or_else(|_| String::new());
    loop {
        match clipboard.get_contents() {
            Ok(contents) => {
                // Empty clipboard (Linux)
                if contents.is_empty() {
                    continue;
                };

                // Clipboard changed
                if contents != last_contents {
                    last_contents = contents.clone();

                    if let Ok(mut url) = Url::parse(&contents) {
                        // Skip URLs without a host
                        let url_inner = url.clone();
                        let Some(host) = url_inner.host_str() else { continue };

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

                        // Update clipboard
                        clipboard.set_contents(url.clone()).map_err(|e| {
                            miette!(format!("Couldn't set clipboard contents: {e}"))
                        })?;
                        last_contents = url;
                    };
                }
            }
            // Empty clipboard (Mac, Windows)
            Err(_) => continue,
        };
    }
}
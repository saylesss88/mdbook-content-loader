//! # mdbook-content-loader
//!
//! An mdBook preprocessor that loads external content and injects it into the book
//! context. This serves as a foundation for the Kanagawa theme's dynamic data loading.
use mdbook_content_loader::ContentLoader;
use mdbook_preprocessor::{MDBOOK_VERSION, Preprocessor, errors::Error};
use semver::{Version, VersionReq};
use std::io;
use std::process;

fn main() {
    // Initialize logging via env_logger (usually controlled by RUST_LOG environment variable)    env_logger::init();

    let preprocessor = ContentLoader::new();
    let args: Vec<String> = std::env::args().collect();

    // 1. Handle version flags
    if args.get(1).map(String::as_str) == Some("--version")
        || args.get(1).map(String::as_str) == Some("-V")
    {
        println!("mdbook-content-loader {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // 2. Respond to mdBook's renderer support check
    // mdBook calls `mdbook-content-loader supports <renderer-name>` first.
    if args.get(1).map(String::as_str) == Some("supports") {
        // We currently support all renderers (html, pdf, epub, etc.)
        process::exit(0);
    }

    // 3. Main execution loop
    if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{e}");
        process::exit(1);
    }
}

/// Handles the standard mdBook preprocessor protocol.
///
/// This involves:
/// 1. Parsing the JSON input from `stdin` into a Context and Book.
/// 2. Checking for version compatibility between the plugin and mdBook.
/// 3. Running the actual transformation logic.
/// 4. Writing the modified Book back to `stdout` as JSON.
///
/// # Errors
///
/// Returns an [`mdbook_preprocessor::errors::Error`] if:
/// * The input from `stdin` is malformed JSON.
/// * The mdBook version string fails to parse.
/// * The [`Preprocessor::run`] implementation encounters an internal failure.
fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    // Standard mdBook protocol: read context and book from stdin
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    // Validate version compatibility
    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        log::warn!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    // Execute the transformation logic
    let processed_book = pre.run(&ctx, book)?;

    // Send the results back to mdbook via stdout
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

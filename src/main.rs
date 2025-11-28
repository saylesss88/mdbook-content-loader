use mdbook_content_loader::ContentLoader;
use mdbook_preprocessor::{errors::Error, Preprocessor, MDBOOK_VERSION};
use semver::{Version, VersionReq};
use std::io;
use std::process;

fn main() {
    env_logger::init();
    let preprocessor = ContentLoader::new();

    // Minimal args handling like your mdbook-content-collections crate
    let args: Vec<String> = std::env::args().collect();

    // Optional: simple version flag
    if args.get(1).map(|s| s.as_str()) == Some("--version")
        || args.get(1).map(|s| s.as_str()) == Some("-V")
    {
        println!("mdbook-content-loader {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // mdBook calls `mdbook-<name> supports <renderer>` first.
    if args.get(1).map(|s| s.as_str()) == Some("supports") {
        // Optional: if you only want HTML, you can check args.get(2)
        // and decide whether to exit 0 or 1.
        //
        // let renderer = args.get(2).map(|s| s.as_str()).unwrap_or("html");
        // if renderer == "html" { process::exit(0); } else { process::exit(1); }

        // For now, claim support for all renderers.
        process::exit(0);
    }

    if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    // New: use mdbook_preprocessor::parse_input instead of CmdPreprocessor
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

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

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

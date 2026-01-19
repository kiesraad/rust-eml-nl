use std::{ffi::OsStr, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use eml_nl::{
    documents::EML,
    io::{EMLParsingMode, EMLRead as _, EMLWrite as _},
};
use sha2::{Digest as _, Sha256};
use tokio::io::AsyncReadExt;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the EML file to validate
    path: PathBuf,

    /// Whether to use strict parsing where no value parse errors are tolerated
    #[arg(long, default_value_t = false)]
    strict: bool,

    /// Print debug representation of EML document
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Whether to output the parsed EML document back to XML
    #[arg(long, default_value_t = false)]
    print: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::try_parse().context("Failed to parse command line arguments")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let file_content = if args.path == OsStr::new("-") {
        info!("Reading EML file as UTF-8 from stdin");
        let mut data = String::new();
        tokio::io::stdin()
            .read_to_string(&mut data)
            .await
            .context("Failed to read EML file from stdin")?;
        data
    } else {
        info!("Reading EML file as UTF-8 from {:?}", args.path);
        tokio::fs::read_to_string(&args.path)
            .await
            .context("Failed to read EML file")?
    };

    info!(
        "Successfully read EML file, size: {} bytes",
        file_content.len()
    );

    info!("Computing SHA-256 hash of the EML file");
    let digest = Sha256::digest(file_content.as_bytes());
    let hex = format!("{:x}", digest);

    info!(
        "SHA-256 hash: {}",
        hex.as_bytes()
            .chunks(4)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join(" ")
    );

    info!("Parsing EML file");

    let (doc, errors) = EML::parse_eml(
        &file_content,
        if args.strict {
            EMLParsingMode::Strict
        } else {
            EMLParsingMode::StrictFallback
        },
    )
    .ok_with_errors()
    .context("Failed to parse EML file")?;

    if errors.is_empty() {
        info!("EML file was parsed succesfully");
    } else {
        info!(
            "EML file was parsed succesfully, but with {} warning(s):",
            errors.len()
        );
        for error in errors {
            match error.span() {
                Some(span) => warn!(" - At position {}: {}", span, error.kind()),
                None => warn!(" - {}", error.kind()),
            }
        }
    }

    info!(
        "Parsed EML document type: {} ({})",
        doc.to_eml_id(),
        doc.to_friendly_name()
    );

    if args.debug {
        info!("Debug representation of parsed EML document:\n{:#?}", doc);
    }

    if args.print {
        info!("Outputting parsed EML document back to XML:");
        let xml = doc
            .write_eml_root_str(true, true)
            .context("Failed to serialize EML document back to XML")?;
        info!("EML XML output:\n{}", xml);
    }

    Ok(())
}

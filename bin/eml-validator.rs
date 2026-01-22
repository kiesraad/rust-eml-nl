use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

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

    let parsing_mode = if args.strict {
        EMLParsingMode::Strict
    } else {
        EMLParsingMode::StrictFallback
    };

    if args.path == OsStr::new("-") {
        info!("Reading EML file as UTF-8 from stdin");
        let mut data = String::new();
        tokio::io::stdin()
            .read_to_string(&mut data)
            .await
            .context("Failed to read EML file from stdin")?;
        handle_file(&data, parsing_mode, args.print, args.debug).await?;
    } else {
        if args.path.is_dir() {
            info!("EML path is a directory, processing all .eml.xml files inside recursively");
            let eml_files = collect_eml_files(&args.path).await?;
            info!("Found {} EML files to process", eml_files.len());
            let mut results = vec![];
            for eml_file in eml_files {
                info!("Processing EML file {:?}", eml_file);
                results.push(process_file_and_log_errors(&eml_file).await);
            }
            info!("Finished processing all EML files");
            info!(
                "Found {} files that parsed successfully without warnings",
                results
                    .iter()
                    .filter(|r| matches!(r, ProcessResult::Success))
                    .count()
            );
            info!(
                "Found {} files that parsed with warnings",
                results
                    .iter()
                    .filter(|r| matches!(r, ProcessResult::WithWarnings(_)))
                    .count()
            );
            info!(
                "Found {} files that failed to parse",
                results
                    .iter()
                    .filter(|r| matches!(r, ProcessResult::Error))
                    .count()
            );
        } else {
            info!("Reading EML file as UTF-8 from {:?}", args.path);
            let content = tokio::fs::read_to_string(&args.path)
                .await
                .context("Failed to read EML file")?;
            handle_file(&content, parsing_mode, args.print, args.debug).await?;
        }
    }

    Ok(())
}

enum ProcessResult {
    Success,
    WithWarnings(usize),
    Error,
}

async fn process_file_and_log_errors(file: impl AsRef<Path>) -> ProcessResult {
    let path = file.as_ref();
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            match handle_file(&content, EMLParsingMode::StrictFallback, false, false).await {
                Ok(warnings) => {
                    if warnings == 0 {
                        ProcessResult::Success
                    } else {
                        ProcessResult::WithWarnings(warnings)
                    }
                }
                Err(e) => {
                    warn!("Error processing file {:?}: {:?}", path, e);
                    ProcessResult::Error
                }
            }
        }
        Err(e) => {
            warn!("Error reading file {:?}: {:?}", path, e);
            ProcessResult::Error
        }
    }
}

async fn collect_eml_files(dir: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    let mut eml_files = Vec::new();
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .context(format!("Failed to read directory {:?}", dir))?;
    while let Some(entry) = entries
        .next_entry()
        .await
        .context("Failed to read directory entry")?
    {
        let path = entry.path();
        if path.is_dir() {
            let mut nested_files = Box::pin(collect_eml_files(&path)).await?;
            eml_files.append(&mut nested_files);
        } else if let Some(filename) = path.file_name() {
            if filename.to_string_lossy().ends_with(".eml.xml") {
                eml_files.push(path);
            }
        }
    }
    Ok(eml_files)
}

async fn handle_file(
    file_content: &str,
    parsing_mode: EMLParsingMode,
    print: bool,
    debug: bool,
) -> anyhow::Result<usize> {
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
    let (doc, errors) = EML::parse_eml(&file_content, parsing_mode)
        .ok_with_errors()
        .context("Failed to parse EML file")?;

    if errors.is_empty() {
        info!("EML file was parsed succesfully");
    } else {
        info!(
            "EML file was parsed succesfully, but with {} warning(s):",
            errors.len()
        );
        for error in &errors {
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

    if debug {
        info!("Debug representation of parsed EML document:\n{:#?}", doc);
    }

    if print {
        info!("Outputting parsed EML document back to XML:");
        let xml = doc
            .write_eml_root_str(true, true)
            .context("Failed to serialize EML document back to XML")?;
        info!("EML XML output:\n{}", xml);
    }

    Ok(errors.len())
}

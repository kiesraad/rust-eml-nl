use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, error::ErrorKind};
use eml_nl::{
    csv::find_matching_documents,
    documents::EML,
    io::{EMLParsingMode, EMLRead as _},
};
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

/// Arguments for the eml2csv CLI tool
#[derive(Debug, Parser)]
#[command(version, about = "Convert EML election data to CSV format (osv4-3)")]
struct Cli {
    /// Path to the EML-510b counting file
    counts_eml: PathBuf,

    /// Path to the EML-230b candidates file
    candidates_eml: PathBuf,

    /// Path for the output CSV file (auto-generated if not provided). Pass '-' to output to stdout.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Whether to disable the UTF-8 BOM in the output CSV file
    #[arg(long = "no-bom", action = clap::ArgAction::SetFalse, default_value_t = true)]
    bom: bool,

    /// Whether to include a trailing newline at the end of the output CSV file
    #[arg(long)]
    trailing_newline: bool,

    /// Do not output any logging to stderr. Will be overridden by the EML_LOG environment variable.
    #[arg(long)]
    quiet: bool,

    /// Be verbose about logging output. Will be overridden by the EML_LOG environment variable.
    #[arg(long)]
    verbose: bool,
}

/// Main entry point
fn main() -> anyhow::Result<()> {
    // Parse command line arguments (help and version should not give an error)
    let args = match Cli::try_parse() {
        Ok(args) => args,
        Err(e) => {
            if e.kind() == ErrorKind::DisplayHelp || e.kind() == ErrorKind::DisplayVersion {
                eprintln!("{}", e);
                std::process::exit(0);
            } else {
                return Err(e).context("Failed to parse command line arguments");
            }
        }
    };

    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_env_var("EML_LOG")
                .with_default_directive(if args.quiet {
                    LevelFilter::OFF.into()
                } else if args.verbose {
                    LevelFilter::DEBUG.into()
                } else {
                    LevelFilter::INFO.into()
                })
                .from_env_lossy(),
        )
        .with_writer(std::io::stderr)
        .init();

    let (output, output_path) = process(
        args.counts_eml,
        args.candidates_eml,
        args.output,
        args.bom,
        args.trailing_newline,
    )?;

    // check if the output path is a dash, then we output to stdout instead of a file
    if output_path == Path::new("-") {
        info!("Writing output to stdout");
        println!("{}", output);
    } else {
        info!("Writing output to file: {}", output_path.display());
        std::fs::write(&output_path, output.as_bytes())
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;
    }

    info!("Output written");

    Ok(())
}

/// Processing function for converting EML files to CSV format
fn process(
    counts_eml: impl AsRef<Path>,
    candidates_eml: impl AsRef<Path>,
    output_path: Option<impl AsRef<Path>>,
    include_bom: bool,
    trailing_newline: bool,
) -> anyhow::Result<(String, PathBuf)> {
    // Load and parse both EML files
    let first_xml = load_and_parse(counts_eml)?;
    let second_xml = load_and_parse(candidates_eml)?;

    info!("Starting conversion to OSV4-3 CSV file");

    // Find the relevant documents and contests in both files, ensuring they match
    let (counts, candidates) = find_matching_documents(&first_xml, &second_xml)?;

    // Determine output path: use provided path or auto-generate based on metadata
    let output_path = output_path.map_or_else(
        || counts.as_osv4_3_csv_filename(),
        |v| Ok(v.as_ref().into()),
    )?;

    // Generate the CSV output
    let output = counts.as_osv4_3_csv(candidates, include_bom, trailing_newline)?;

    info!("Processing completed");

    Ok((output, output_path))
}

/// Load and parse an EML file from the given path, returning the EML document
/// or an error with context if it fails.
fn load_and_parse(path: impl AsRef<Path>) -> Result<EML, anyhow::Error> {
    info!("Loading EML file: {}", path.as_ref().display());
    let xml = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read EML file: {}", path.as_ref().display()))?;
    debug!("Successfully read EML file, size: {} bytes", xml.len());
    debug!("Parsing EML file");
    let eml = EML::parse_eml(&xml, EMLParsingMode::Strict)
        .ok()
        .with_context(|| format!("Failed to parse file as EML: {}", path.as_ref().display()))?;
    info!(
        "EML file was parsed succesfully, found document type: {}",
        eml.to_eml_id()
    );
    Ok(eml)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_error_contains {
        ($err:expr, $expected:expr $(,)?) => {{
            let error = &$err;
            let expected = $expected;
            let found = error.chain().any(|e| e.to_string().contains(expected));

            if !found {
                let chain_str: Vec<String> = error.chain().map(|e| format!("  - {}", e)).collect();
                panic!(
                    "Assertion failed: error chain did not contain \"{}\"\nFull error chain:\n{}",
                    expected,
                    chain_str.join("\n")
                );
            }
        }};
    }

    #[test]
    fn test_only_provide_candidate_lists() {
        let err = process(
            "test-files/csv/Kandidatenlijsten_GR2022_Groningen.eml.xml",
            "test-files/csv/Kandidatenlijsten_GR2022_Groningen.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap_err();
        assert_error_contains!(
            err,
            "EML-230b candidates document found, but missing EML-510 counts document"
        );
    }

    #[test]
    fn test_only_provide_counts() {
        let err = process(
            "test-files/csv/Telling_GR2022_Groningen.eml.xml",
            "test-files/csv/Telling_GR2022_Groningen.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap_err();
        assert_error_contains!(
            err,
            "EML-510 counts document found, but missing EML-230b candidate lists document"
        );
    }

    #[test]
    fn test_provide_files_from_different_elections() {
        let err = process(
            "test-files/csv/Kandidatenlijsten_GR2022_WestMaasenWaal.eml.xml",
            "test-files/csv/Telling_GR2022_Groningen.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap_err();
        assert_error_contains!(
            err,
            "Election id of counts file ('GR2022_Groningen') does not match candidate lists id ('GR2022_WestMaasenWaal')"
        );
    }

    #[test]
    fn test_provide_wrong_candidate_list() {
        let err = process(
            "test-files/csv/Kandidatenlijsten_TK2025_Haarlem.eml.xml",
            "test-files/csv/Telling_TK2025_gemeente_West_Maas_en_Waal.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap_err();
        assert_error_contains!(
            err,
            "Contest id of counts file ('6') does not match candidate lists id ('10')"
        );
    }

    #[test]
    fn test_gr2022_groningen() {
        let res = process(
            "test-files/csv/Kandidatenlijsten_GR2022_Groningen.eml.xml",
            "test-files/csv/Telling_GR2022_Groningen.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap();
        assert_eq!(
            res.0,
            include_str!("../test-files/csv/osv4-3_telling_gr2022_groningen.csv")
        );
    }

    #[test]
    fn test_gr2022_west_maas_en_waal_reverse() {
        let res = process(
            "test-files/csv/Telling_GR2022_WestMaasenWaal.eml.xml",
            "test-files/csv/Kandidatenlijsten_GR2022_WestMaasenWaal.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap();
        assert_eq!(
            res.0,
            include_str!("../test-files/csv/osv4-3_telling_gr2022_westmaasenwaal.csv")
        );
    }
}

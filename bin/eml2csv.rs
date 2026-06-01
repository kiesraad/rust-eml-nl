use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, error::ErrorKind};
use eml_nl::{
    documents::{EML, candidate_lists::CandidateLists, election_count::ElectionCount},
    io::{EMLParsingMode, EMLRead as _},
};
use tracing::{debug, error, info, level_filters::LevelFilter};
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

/// Find the relevant documents and contests in both EML files, ensuring they match.
fn find_matching_documents<'a>(
    first_xml: &'a EML,
    second_xml: &'a EML,
) -> Result<(&'a ElectionCount, &'a CandidateLists), anyhow::Error> {
    // Determine which file is counts and which is candidates based on document type
    let (counts, candidates) = if let Some(election_count) = first_xml.as_count_doc() {
        debug!("First file is identified as counts document");
        if let Some(candidate_lists) = second_xml.as_candidate_lists_doc() {
            debug!("Second file is identified as candidates document");
            (election_count, candidate_lists)
        } else {
            error!("Second file does not contain a valid EML-230b candidates document");
            anyhow::bail!(
                "I got an EML-510 counts document, but you did not provide a valid EML-230b candidates file"
            );
        }
    } else if let Some(candidate_lists) = first_xml.as_candidate_lists_doc() {
        debug!("First file is identified as candidates document");
        if let Some(election_count) = second_xml.as_count_doc() {
            debug!("Second file is identified as counts document");
            (election_count, candidate_lists)
        } else {
            error!("Second file does not contain a valid EML-510 counts document");
            anyhow::bail!(
                "I got an EML-230b candidates document, but you did not provide a valid EML-510 counts file"
            );
        }
    } else {
        error!("Neither file provided contains a valid counts or candidates document");
        anyhow::bail!(
            "You must provide a valid EML-510 counts file and a valid EML-230b candidates file"
        );
    };

    // Make sure both files are talking about the same election
    let counts_election_id = &counts.count.election.identifier.id;
    let candidates_election_id = &candidates.candidate_list.election.identifier.id;
    if counts_election_id != candidates_election_id {
        error!("Failed to match election ids of documents provided");
        anyhow::bail!(
            "Election ids of files provided do not match: '{}' vs '{}'",
            counts_election_id.raw(),
            candidates_election_id.raw()
        );
    }

    // Extract the contests from both files
    let count_contest = counts
        .count
        .election
        .contests
        .first()
        .context("No contests found in counts file")?;
    let candidates_contest = candidates
        .candidate_list
        .election
        .contests
        .first()
        .context("No contests found in candidates file")?;

    // Make sure both files are talking about the same contest
    if count_contest.identifier.id.raw() != candidates_contest.identifier.id.raw() {
        error!("Failed to match contest ids of documents provided");
        anyhow::bail!(
            "Contest ids of files provided do not match: '{}' vs '{}'",
            count_contest.identifier.id.raw(),
            candidates_contest.identifier.id.raw()
        );
    }

    Ok((counts, candidates))
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
            "I got an EML-230b candidates document, but you did not provide a valid EML-510 counts file"
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
            "I got an EML-510 counts document, but you did not provide a valid EML-230b candidates file"
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
            "Election ids of files provided do not match: 'GR2022_Groningen' vs 'GR2022_WestMaasenWaal'"
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
            "Contest ids of files provided do not match: '6' vs '10'"
        );
    }

    #[test]
    fn test_gr2022_groningen() {
        let err = process(
            "test-files/csv/Kandidatenlijsten_GR2022_Groningen.eml.xml",
            "test-files/csv/Telling_GR2022_Groningen.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap();
        assert_eq!(
            err.0,
            include_str!("../test-files/csv/osv4-3_telling_gr2022_groningen.csv")
        );
    }

    #[test]
    fn test_gr2022_west_maas_en_waal_reverse() {
        let err = process(
            "test-files/csv/Telling_GR2022_WestMaasenWaal.eml.xml",
            "test-files/csv/Kandidatenlijsten_GR2022_WestMaasenWaal.eml.xml",
            None::<PathBuf>,
            true,
            false,
        )
        .unwrap();
        assert_eq!(
            err.0,
            include_str!("../test-files/csv/osv4-3_telling_gr2022_westmaasenwaal.csv")
        );
    }
}

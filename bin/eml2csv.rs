use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use clap::{Parser, error::ErrorKind};
use eml_nl::{
    documents::{
        EML,
        candidate_lists::{CandidateLists, CandidateListsContest},
        election_count::{
            CountType, ElectionCount, ElectionCountContest, RejectedVotesReason,
            UncountedVotesReason,
        },
    },
    io::{EMLParsingMode, EMLRead as _},
    utils::{AffiliationId, CandidateId, ElectionCategory, ElectionId, StringValueData as _},
};
use regex::Regex;
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

    process(
        args.counts_eml,
        args.candidates_eml,
        args.output,
        args.bom,
        args.trailing_newline,
    )
}

/// Regular expression to extract postal code from polling station
static POSTCODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(postcode: (\d{4}\s*\w{2})\)").expect("Failed to compile postcode regex")
});

/// Regular expression to clean stembureau id prefixes
static SB_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+::SB").expect("Failed to compile stembureau id regex"));

/// List of special municipalities that should be labelled as "Openbaar lichaam" instead of "Gemeente"
static SPECIAL_MUNICIPALITIES: &[&str] = &["Bonaire", "Saba", "Sint Eustatius"];

/// Processing function for converting EML files to CSV format
fn process(
    counts_eml: PathBuf,
    candidates_eml: PathBuf,
    output_file: Option<PathBuf>,
    include_bom: bool,
    trailing_newline: bool,
) -> anyhow::Result<()> {
    // Load and parse both EML files
    let first_xml = load_and_parse(&counts_eml)?;
    let second_xml = load_and_parse(&candidates_eml)?;

    info!("Starting conversion to osv4-3 CSV file");

    // Find the relevant documents and contests in both files, ensuring they match
    let (counts, count_contest, _, candidates_contest) =
        find_matching_documents(&first_xml, &second_xml)?;

    // Election metadata
    let election_identifier = &counts.count.election.identifier;
    let authority_identifier = &counts.managing_authority.authority_identifier;
    let election_id = election_identifier
        .id
        .cloned_value()
        .context("Could not process election id")?;
    let election_name = election_identifier.name.as_deref().unwrap_or("");
    let election_date = election_identifier
        .election_date
        .copied_value()
        .context("Could not process election date")?;
    let election_category = election_identifier
        .category
        .copied_value()
        .context("Could not process election category")?;
    let authority_name = authority_identifier.name.as_deref().unwrap_or("");
    let authority_id = authority_identifier
        .id
        .cloned_value()
        .context("Could not process authority id")?;
    let authority_type = if SPECIAL_MUNICIPALITIES.contains(&authority_name) {
        "Openbaar lichaam"
    } else {
        "Gemeente"
    };

    info!("Processing for election {:?}", election_id);

    let total_votes = count_contest
        .total_votes
        .as_ref()
        .context("No total votes found in counts file")?;

    // Extract polling station information from the count contest, including cleaned names and postal codes
    let stations = extract_polling_stations(count_contest)?;

    // Build CSV output
    let mut output = Output::new(include_bom);

    // Header block (rows 1-4)
    output.row(["Verkiezing", "", election_name]);
    output.row(["Datum", "", &election_date.to_raw_value()]);
    output.row([
        "Gebied",
        "",
        &format!("{} {}", authority_type, authority_name),
    ]);
    output.row(["Nummber", "", authority_id.value()]);

    // row 5: empty separator row
    output.empty_row();

    // Column header rows (6-8)
    output.row(
        [
            "Lijstnummer",
            "Aanduiding",
            "Volgnummer",
            "Naam kandidaat",
            "Totaal",
        ]
        .into_iter()
        .chain(stations.iter().map(|s| s.cleaned_name.as_str())),
    );
    output.row(
        ["Gebiednummer", "", "", "", ""]
            .into_iter()
            .chain(stations.iter().map(|s| s.plain_ps_id.as_str())),
    );
    output.row(
        ["Postcode", "", "", "", ""]
            .into_iter()
            .chain(stations.iter().map(|s| s.postal_code.as_str())),
    );

    // Rows about various totals (starting from row 9)
    let reporting_units = &count_contest.reporting_unit_votes;
    fn stat_row_start<'a>(
        label: &'static str,
        total_value: impl Into<Cow<'a, str>>,
    ) -> impl Iterator<Item = Cow<'a, str>> {
        vec![
            Cow::Borrowed(""),
            Cow::Borrowed(label),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            total_value.into(),
        ]
        .into_iter()
    }

    // Helper macro to reduce repetition for various totals rows
    macro_rules! stat_row {
        ($label:expr, $votes_type:ident, $filter_type:expr) => {
            stat_row_start(
                $label,
                total_votes
                    .$votes_type
                    .get(&$filter_type)
                    .map(|v| v.raw())
                    .unwrap_or(Cow::Borrowed("0")),
            )
            .chain(reporting_units.iter().map(|r| {
                r.$votes_type
                    .get(&$filter_type)
                    .map(|v| v.raw())
                    .unwrap_or(Cow::Borrowed("0"))
            }))
        };
    }

    output.row(
        stat_row_start("opgeroepenen", total_votes.eligible_voter_count.raw())
            .chain(reporting_units.iter().map(|r| r.eligible_voter_count.raw())),
    );

    output.row(stat_row!(
        "geldige stempas",
        uncounted_votes,
        UncountedVotesReason::ValidPollCards
    ));

    output.row(stat_row!(
        "geldig volmachtbewijs",
        uncounted_votes,
        UncountedVotesReason::ValidProxyCertificates
    ));

    output.row(stat_row!(
        "geldige kiezerspas",
        uncounted_votes,
        UncountedVotesReason::ValidVoterCards
    ));

    output.row(stat_row!(
        "toegelaten kiezers",
        uncounted_votes,
        UncountedVotesReason::AdmittedVoters
    ));

    output.row(
        stat_row_start(
            "geldige stembiljetten",
            total_votes.candidate_votes_count.raw(),
        )
        .chain(
            reporting_units
                .iter()
                .map(|r| r.candidate_votes_count.raw()),
        ),
    );

    output.row(stat_row!(
        "blanco stembiljetten",
        rejected_votes,
        RejectedVotesReason::Blank
    ));

    output.row(stat_row!(
        "ongeldige stembiljetten",
        rejected_votes,
        RejectedVotesReason::Invalid
    ));

    let counted_ballots = (total_votes.blank_votes()?.copied_value()?
        + total_votes.invalid_votes()?.copied_value()?
        + total_votes.candidate_votes_count.copied_value()?)
    .to_string();
    let reporting_unit_counted_ballots = reporting_units
        .iter()
        .map(|r| -> Result<Cow<str>, _> {
            Ok(Cow::Owned(
                (r.blank_votes()?.copied_value()?
                    + r.invalid_votes()?.copied_value()?
                    + r.candidate_votes_count.copied_value()?)
                .to_string(),
            ))
        })
        .collect::<Result<Vec<Cow<str>>, anyhow::Error>>()?;
    output.row(
        stat_row_start("aangetroffen stembiljetten", counted_ballots)
            .chain(reporting_unit_counted_ballots),
    );

    output.row(stat_row!(
        "meer stembiljetten dan toegelaten kiezers",
        uncounted_votes,
        UncountedVotesReason::MoreBallotsCounted
    ));

    output.row(stat_row!(
        "minder stembiljetten dan toegelaten kiezers",
        uncounted_votes,
        UncountedVotesReason::FewerBallotsCounted
    ));

    output.row(stat_row!(
        "kiezers met stembiljet hebben niet gestemd",
        uncounted_votes,
        UncountedVotesReason::BallotsTaken
    ));

    output.row(stat_row!(
        "er zijn te weinig stembiljetten uitgereikt",
        uncounted_votes,
        UncountedVotesReason::TooFewBallotsIssued
    ));

    output.row(stat_row!(
        "er zijn te veel stembiljetten uitgereikt",
        uncounted_votes,
        UncountedVotesReason::TooManyBallotsIssued
    ));

    output.row(stat_row!(
        "geen verklaring",
        uncounted_votes,
        UncountedVotesReason::NoExplanation
    ));

    output.row(stat_row!(
        "andere verklaring",
        uncounted_votes,
        UncountedVotesReason::OtherExplanation
    ));

    // Build candidate name map: (aff_id, cand_id) -> "lastname, initials" (with optional prefix)
    let mut candidate_names: HashMap<(AffiliationId, CandidateId), String> = HashMap::new();
    for affiliation in &candidates_contest.affiliations {
        let aff_id = affiliation.identifier.id.cloned_value()?;
        for candidate in &affiliation.candidates {
            let cand_id = candidate.identifier.id.cloned_value()?;
            let pn = &candidate.full_name.person_name;
            let last = pn.get_last_name();
            let prefix = pn.get_name_prefix();
            let initials = pn.get_initials();

            let last_part = if let Some(prefix) = prefix {
                format!("{} {}", prefix, last)
            } else {
                last.to_string()
            };

            let name = if let Some(initials) = initials {
                format!("{}, {}", last_part, initials)
            } else {
                last_part
            };

            candidate_names.insert((aff_id, cand_id), name);
        }
    }

    // Build a per-station lookup: for each station, a map of (aff_id, Option<cand_id>) -> votes
    let station_maps: Vec<HashMap<(AffiliationId, Option<CandidateId>), u64>> = count_contest
        .reporting_unit_votes
        .iter()
        .map(|ru| {
            let sels = ru
                .selections_per_affiliation()
                .context("Failed to extract selections from reporting unit votes")?;
            let mut map = HashMap::new();
            for av in &sels {
                let aff_id = av.affiliation.id.cloned_value()?;
                map.insert((aff_id, None), av.valid_votes);
                for cv in &av.candidates {
                    let cand_id = cv.candidate.identifier.id.cloned_value()?;
                    map.insert((aff_id, Some(cand_id)), cv.valid_votes);
                }
            }
            Ok(map)
        })
        .collect::<anyhow::Result<_>>()?;

    // Get ordered affiliation/candidate data from total votes (already parsed u64 values)
    let total_selections = total_votes
        .selections_per_affiliation()
        .context("Failed to extract selections from total votes")?;

    // Vote data rows (parties and candidates in document order from total_votes)
    for av in &total_selections {
        let aff_id = av.affiliation.id.cloned_value()?;
        let aff_name = &av.affiliation.name[..];

        // output row for affiliation
        output.row(
            [
                Cow::Owned(aff_id.to_raw_value().into()),
                Cow::Borrowed(aff_name),
                Cow::Borrowed(""),
                Cow::Borrowed(""),
                Cow::Owned(av.valid_votes.to_string()),
            ]
            .into_iter()
            .chain(station_maps.iter().map(|sm| {
                sm.get(&(aff_id, None))
                    .copied()
                    .unwrap_or(0)
                    .to_string()
                    .into()
            })),
        );

        for cv in &av.candidates {
            let cand_id = cv.candidate.identifier.id.cloned_value()?;
            let cand_name = candidate_names
                .get(&(aff_id, cand_id))
                .map(|s| s.as_str())
                .context("Failed to retrieve candidate name")?;

            // output row for candidate
            output.row(
                [
                    Cow::Borrowed(""),
                    Cow::Borrowed(""),
                    Cow::Owned(cand_id.to_raw_value().into()),
                    Cow::Borrowed(cand_name),
                    Cow::Owned(cv.valid_votes.to_string()),
                ]
                .into_iter()
                .chain(station_maps.iter().map(|sm| {
                    sm.get(&(aff_id, Some(cand_id)))
                        .copied()
                        .unwrap_or(0)
                        .to_string()
                        .into()
                })),
            );
        }
    }

    // Remove trailing newline if present
    if !trailing_newline {
        debug!("Outputting without trailing newline");
        output.remove_trailing_newline();
    } else {
        debug!("Including trailing newline in output")
    }

    // Determine output path: use provided path or auto-generate based on metadata
    let output_path = output_file.unwrap_or_else(|| {
        generate_filename(
            authority_name,
            &election_id,
            election_category,
            authority_type,
        )
    });

    // check if the output path is a dash, then we output to stdout instead of a file
    if output_path == Path::new("-") {
        info!("Writing output to stdout");
        println!("{}", output.into_string());
    } else {
        info!("Writing output to file: {}", output_path.display());
        std::fs::write(&output_path, output.into_string().as_bytes())
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;
    }

    info!("Processing completed");

    Ok(())
}

struct CsvPollingStation {
    cleaned_name: String,
    plain_ps_id: String,
    postal_code: String,
}

/// Extract polling station information from the count contest, including cleaned names and postal codes.
fn extract_polling_stations(
    count_contest: &ElectionCountContest,
) -> Result<Vec<CsvPollingStation>, anyhow::Error> {
    count_contest
        .reporting_unit_votes
        .iter()
        .map(|ru| {
            let raw_name = &ru.identifier.name;
            let id = ru
                .identifier
                .id
                .cloned_value()
                .context("Could not retrieve reporting unit identifier")?;

            let postal_code = POSTCODE_RE
                .captures(raw_name)
                .map(|c| c[1].to_string())
                .unwrap_or_default();

            let name = POSTCODE_RE.replace(raw_name, "").to_string();
            let cleaned_name = name
                .strip_prefix("Stembureau ")
                .or_else(|| name.strip_prefix("Briefstembureau "))
                .unwrap_or(&name)
                .to_string();

            let raw_id = id.to_raw_value();
            let plain_ps_id = SB_ID_RE.replace(&raw_id, "").to_string();

            Ok(CsvPollingStation {
                cleaned_name,
                plain_ps_id,
                postal_code,
            })
        })
        .collect()
}

/// Generate an output filename based on the election and authority metadata, if no output path is provided.
fn generate_filename(
    authority_name: &str,
    election_id: &ElectionId,
    election_category: ElectionCategory,
    authority_type: &str,
) -> PathBuf {
    info!("No output path provided, auto-generating filename");
    debug!(
        "Inputs for filename: authority name: '{}', election id: '{:?}', election category: '{:?}', authority type: '{}'",
        authority_name, election_id, election_category, authority_type
    );
    let norm_authority_name = normalise(authority_name);
    let norm_election_id = normalise(&election_id.value()[..6]);
    if election_category == ElectionCategory::GR {
        debug!("Election category is GR, omitting authority type from filename");
        PathBuf::from(format!(
            "osv4-3_telling_{}_{}.csv",
            norm_election_id, norm_authority_name
        ))
    } else {
        debug!("Election category is not GR, including authority type in filename");
        PathBuf::from(format!(
            "osv4-3_telling_{}_{}_{}.csv",
            norm_election_id,
            authority_type.to_lowercase().replace(' ', "_"),
            norm_authority_name
        ))
    }
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
) -> Result<
    (
        &'a ElectionCount,
        &'a ElectionCountContest,
        &'a CandidateLists,
        &'a CandidateListsContest,
    ),
    anyhow::Error,
> {
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

    // Count type must be municipal for this tool to work (EML-510b)
    if counts.count_type != CountType::Municipal {
        error!("The counts file provided is not an EML-510b municipal count document");
        anyhow::bail!(
            "Counts file is not an EML-510b document (got {:?})",
            counts.count_type
        );
    }

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

    Ok((counts, count_contest, candidates, candidates_contest))
}

/// Quote a single CSV field: always wrapped in `"` (unless field is empty), internal `"` doubled.
fn qf(s: &str) -> String {
    if s.is_empty() {
        return "".to_string();
    }
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// Strip non-ASCII-alphanumeric chars and lowercase (for auto-generated filenames).
fn normalise(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

struct Output {
    output_str: String,
}

impl Output {
    /// Create a new Output with an optional UTF-8 BOM at the beginning.
    fn new(include_bom: bool) -> Self {
        let mut output_str = String::new();
        if include_bom {
            output_str.push('\u{FEFF}'); // UTF-8 BOM
        }
        Self { output_str }
    }

    fn row(&mut self, fields: impl IntoIterator<Item = impl AsRef<str>>) {
        let row = fields
            .into_iter()
            .map(|f| qf(f.as_ref()))
            .collect::<Vec<_>>()
            .join(";");
        self.output_str.push_str(&row);
        self.output_str.push('\n');
    }

    fn empty_row(&mut self) {
        self.output_str.push('\n');
    }

    fn remove_trailing_newline(&mut self) {
        if self.output_str.ends_with('\n') {
            self.output_str.pop();
        }
    }

    fn into_string(self) -> String {
        self.output_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postcode_regex_compiles() {
        LazyLock::force(&POSTCODE_RE);
    }

    #[test]
    fn test_sb_id_regex_compiles() {
        LazyLock::force(&SB_ID_RE);
    }
}

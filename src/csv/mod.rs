//! Converting EML files to OSV4-3 CSV format.
//!
//! This module provides functionality to convert an [`ElectionCount`] document
//! into a CSV document following the OSV4-3 format. This format does not have a
//! proper specification and was reverse engineered based on existing files.
//!
//! The main entry point is the [`ElectionCount::as_osv4_3_csv`] method. For
//! information on how to generate CSVs and what is required, please refer to
//! the documentation of that method.
//!
//! ## Format definition
//!
//! The OSV4-3 format broadly consists of a header block, followed by some
//! statistical information, followed by detailed counts for each affiliation
//! and candidate. The format uses a semicolon (`;`) as the separator, uses
//! quotes (`"`) around each field, unless the field is empty. Technically this
//! would make the format more of a *SSV*, but the name *CSV* is more widely
//! known and accepted for formats following this general style.
//!
//! This specific format appears to be an attempt to make importing in Dutch
//! language versions of Excel easy. To facilitate this, the format also emits
//! an UTF-8 BOM as the first three bytes, as without that, Excel would
//! improperly import the file as if it was encoded in *Windows-1252* encoding.
//! Normally the format does not include a final newline.
//!
//! This library does however support disabling the UTF-8 BOM and introducing
//! a final newline in case Excel is not the primary target of the CSV file.
//!
//! The file contents of an OSV4-3 file consist of two blocks. First is a header
//! block defining some properties about the document (explained below). The
//! second block contains detailed information of the counts. This second block
//! consists of three parts. Firstly, two or three header rows. Then a number of
//! (statistical) rows not related to specific candidates or affiliations.
//! Finally rows for each affiliation and each candidate with the number of
//! valid votes for them. A detailed explanation follows below.
//!
//! ### Header block
//!
//! The header block starts with four rows, where the first column in each of
//! these rows defines a property, the second column is always empty, and the
//! third column contains the value for the defined property. The properties
//! defined are: `Verkiezing` (containing the election name), `Datum` (the
//! election date), `Gebied` (containing the name of the area for which the CSV
//! applies, i.e. authority name), and `Nummer` (containing the area code, i.e.
//! authority id).
//!
//! An example header block is:
//!
//! ```csv
//! "Verkiezing";;"Gemeenteraad Roosendaal 2026"
//! "Datum";;"2026-03-18"
//! "Gebied";;"Gemeente Roosendaal"
//! "Nummer";;"1674"
//! ```
//!
//! The header block is always followed by an empty row.
//!
//! ### Content table header rows
//!
//! Following the header block (and an empty row), a list of column names is
//! defined. The initial four columns that always exist are named
//! `Lijstnummer` (AffiliationId), `Aanduiding` (name of the affiliation or
//! statistic), `Volgnummer` (CandidateId) and `Naam kandidaat` (name of the
//! candidate).
//!
//! Following these four, a column `Totaal` defines a column containing the
//! total votes for each following row after the content table header. Note that
//! this column is currently not optional in [`ElectionCount`] documents
//! supported by this library, but the TotalVotes element is optional in some
//! cases.
//!
//! After this, the format contains a column header for each
//! [`ReportingUnitVotes`](crate::documents::election_count::ReportingUnitVotes)
//! in the count (containing the name of the identifier for the reporting unit).
//! For a municipal level document this typically means that each polling
//! station in that municipality will get its own column. For the central count,
//! these are often not contained in the document.
//!
//! The next row always starts with `Gebiednummer` and three empty columns. If
//! the `Totaal` column exists, that column is also left empty for this row.
//! After that each `ReportingUnitVotes` gets its identifier id printed (i.e.
//! typically the polling station numbers).
//!
//! If the count is a municipal count then a third header row exists. This row
//! starts with `Postcode`. This is again followed by at least three empty
//! columns and another empty column if the `Totaal` column exists. After that
//! a postal code is included for each polling station.
//!
//! In order to set the names, ids and postal codes for municipal counts docs,
//! some additional logic is required: the EML format for count documents does
//! not contain nice separated values for each of these. For municipal counts
//! specifically: the postal code is extracted from the identifier name for each
//! polling station. Additionally, the name is then cleanup up to not include
//! that postal code as well as not include any `"Stembureau"` or
//! `"Briefstembureau"` prefixes (these are often duplicated in the EML because
//! municipalities already include the Stembureau prefix in their polling
//! station names, whereas the software also adds that prefix). As an example,
//! a typical reporting unit identifier contains a name looking something like
//! `Stembureau Stembureau Gemeentehuis (postcode: 1234AB)`, where the name of
//! the column header ends up being just `Stembureau Gemeentehuis`. Finally, the
//! id is also modified a little bit, with only the digits of the polling
//! station included (and no `SB` prefix or something similar). Again, note this
//! is only done for municipal counts.
//!
//! As an example, here are the two header rows for a CSB for a municipal
//! election (where there are no reporting units in the EML file):
//!
//! ```csv
//! "Lijstnummer";"Aanduiding";"Volgnummer";"Naam kandidaat";"Totaal"
//! "Gebiednummer";;;;
//! "Postcode";;;;
//! ```
//!
//! And here is a simplied example of the header for a GSB election count
//! (including some polling stations):
//!
//! ```csv
//! "Lijstnummer";"Aanduiding";"Volgnummer";"Naam kandidaat";"Totaal";"Stembureau Huis van Roosendaal";"Stembureau Cultuurhuis De Suite Roosendaal"
//! "Gebiednummer";;;;;"1";"2"
//! "Postcode";;;;;"4701 NK";"4701 EK"
//! ```
//!
//! ### Content table 'statistical' rows
//!
//! Directly after the header a number of 'statistical' rows are included. These
//! can be recognized by containing an empty first column, then the name of some
//! statistic in the second column, and then two empty columns. After that the
//! value (i.e. *count*) for that statistic is included in the `Totaal` column
//! and each reporting unit column. These rows include statistics such as the
//! "toegelaten kiezers" row (number of voters admitted) and
//! "blanco stembiljetten" row (number of blank votes).
//!
//! A few example rows (showing the numbers for a `Totaal` column and a single
//! reporting unit after that in this case):
//!
//! ```csv
//! ;"opgeroepenen";;;"63628";"1595"
//! ;"geldige stempas";;;"23715";"968"
//! ;"geldig volmachtbewijs";;;"3023";"111"
//! ;"toegelaten kiezers";;;"26738";"1079"
//! ;"geldige stembiljetten";;;"26588";"1067"
//! ```
//!
//! ### Content table valid votes rows
//!
//! Directly after the statistical rows, the valid votes per affiliation and
//! candidate follow. Rows for affiliations can be recognized by starting with
//! an affiliation id (i.e. list number) in the first column, and an affiliation
//! name (i.e. political group name) in the second column, followed by two empty
//! columns.
//!
//! Candidate rows can be identified by starting with two empty columns,
//! followed by the candidate id (i.e. candidate number) in the third column,
//! and the candidate name in the fourth column. Note that candidate names have
//! a specific formatting of `{prefix} {last name}, {initials}`.
//!
//! An affiliation row is always directly followed by the candidate rows for
//! that specific affiliation. Once a new affiliation row starts, any following
//! candidate rows will be for that new affiliation.
//!
//! All candidate and affiliation rows are then followed by the number of valid
//! votes in the `Totaal` column, and then for each reporting unit following
//! that.
//!
//! A few example rows (showing the numbers for a `Totaal` column and a single
//! reporting unit after that in this case):
//!
//! ```csv
//! "1";"Testpartij";;;"2234";"145"
//! ;;"1";"van de Kandidaat, A.B.C.";"1200";"111";"34"
//! ;;"2";"Achternaam, V.N.";"105";"20"
//! ;;"3";"Kan, D.";"247";"5";"4"
//! "2";"Andere Partij";;;"3911";"176"
//! ;;"1";"Jansen, J.J.";"1828";"101";"64"
//! ;;"2";"Testkandidaat, T.";"395";"20"
//! ;;"3";"van Tests, S.T.";"247";"5";"4"
//! ```
//!
//! ### Full example
//!
//! Combining all the fragments from before, a full OSV4-3 CSV document might
//! look like this:
//!
//! ```csv
//! "Verkiezing";;"Gemeenteraad Roosendaal 2026"
//! "Datum";;"2026-03-18"
//! "Gebied";;"Gemeente Roosendaal"
//! "Nummer";;"1674"
//!
//! "Lijstnummer";"Aanduiding";"Volgnummer";"Naam kandidaat";"Totaal";"Stembureau Huis van Roosendaal"
//! "Gebiednummer";;;;;"1"
//! "Postcode";;;;;"4701 NK"
//! ;"opgeroepenen";;;"63628";"1595"
//! ;"geldige stempas";;;"23715";"968"
//! ;"geldig volmachtbewijs";;;"3023";"111"
//! ;"toegelaten kiezers";;;"26738";"1079"
//! ;"geldige stembiljetten";;;"26588";"1067"
//! "1";"Testpartij";;;"2234";"145"
//! ;;"1";"van de Kandidaat, A.B.C.";"1200";"111";"34"
//! ;;"2";"Achternaam, V.N.";"105";"20"
//! ;;"3";"Kan, D.";"247";"5";"4"
//! "2";"Andere Partij";;;"3911";"176"
//! ;;"1";"Jansen, J.J.";"1828";"101";"64"
//! ;;"2";"Testkandidaat, T.";"395";"20"
//! ;;"3";"van Tests, S.T.";"247";"5";"4"
//! ```
//!
//! Note that this example includes a final newline and no UTF-8 BOM, whereas an
//! actual exported CSV by default will not include a final newline, but will
//! include an UTF-8 BOM.
//!
//! ### Filename convention
//!
//! By convention, files converted from the ElectionCount EML document are named
//! as follows: given a count document named `Telling_GR2022_Groningen.eml.xml`
//! the associated OSV4-3 CSV file should be named
//! `osv4-3_telling_gr2022_groningen.csv`. Note how the CSV is all lowercase.
//! The CSV file should start with the prefix `osv4-3_telling_`, followed by the
//! lowercase election category and year of the election date, followed by an
//! underscore (`_`) and then followed by the name of the authority (all in
//! lower case again).

mod matcher;
mod name_resolver;
mod utils;
mod writer;

use std::{borrow::Cow, path::PathBuf};

pub use matcher::{EMLMatchError, find_matching_documents};
pub use name_resolver::NameResolver;
use tracing::{debug, info};
pub use writer::CsvWriter;

use crate::{
    EMLError, EMLErrorKind,
    csv::utils::{AffiliationWithVotes, extract_vote_counts},
    documents::election_count::{
        CountType, ElectionCount, RejectedVotesReason, TotalVotes, UncountedVotesReason,
    },
    utils::{ElectionCategory, StringValueData},
};

/// List of special municipalities that should be labelled as "Openbaar lichaam" instead of "Gemeente"
static SPECIAL_MUNICIPALITIES: &[&str] = &["Bonaire", "Saba", "Sint Eustatius"];

/// Strip non-ASCII-alphanumeric chars and lowercase (for auto-generated filenames).
fn normalise(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

impl ElectionCount {
    /// Get a filename suitable for the OSV4-3 file
    pub fn as_osv4_3_csv_filename(&self) -> Result<PathBuf, EMLError> {
        let election_identifier = &self.count.election.identifier;
        let election_id = election_identifier.id.cloned_value()?;
        let election_category = election_identifier.category.copied_value()?;
        let election_date = election_identifier.election_date.copied_value()?;
        let authority_identifier = &self.managing_authority.authority_identifier;
        let authority_name = authority_identifier.name.as_deref().unwrap_or("");
        let authority_type = if self.count_type == CountType::Municipal {
            if SPECIAL_MUNICIPALITIES.contains(&authority_name) {
                "Openbaar lichaam"
            } else {
                "Gemeente"
            }
        } else {
            ""
        };
        debug!(
            "Inputs for filename: authority name: '{}', election id: '{:?}', election category: '{:?}', election date: '{:?}', authority type: '{}'",
            authority_name, election_id, election_category, election_date, authority_type
        );
        let norm_authority_name = normalise(authority_name);
        let base_election_id_value = election_id.value();
        let norm_election_id = if base_election_id_value.len() >= 6 {
            normalise(&base_election_id_value[..6])
        } else {
            // backup in case election id is unexpectedly short
            use chrono::Datelike as _;
            normalise(&format!(
                "{}{}",
                election_category.to_eml_value(),
                election_date.date.year(),
            ))
        };
        if election_category == ElectionCategory::GR {
            debug!("Election category is GR, omitting authority type from filename");
            Ok(PathBuf::from(format!(
                "osv4-3_telling_{}_{}.csv",
                norm_election_id, norm_authority_name
            )))
        } else {
            debug!("Election category is not GR, including authority type in filename");
            Ok(PathBuf::from(format!(
                "osv4-3_telling_{}_{}_{}.csv",
                norm_election_id,
                authority_type.to_lowercase().replace(' ', "_"),
                norm_authority_name
            )))
        }
    }

    /// Convert the ElectionCount to a CSV string in the OSV4-3 format.
    ///
    /// The OSV4-3 format prints the names of candidates and affiliations in
    /// certain rows, but this data is not available in the ElectionCount
    /// document. To deal with this, this method takes a [`NameResolver`]
    /// implementation. The eml2csv binary uses a matching
    /// [`CandidateLists`](crate::documents::candidate_lists::CandidateLists)
    /// document for this. Note that the caller needs to make sure that these
    /// documents are for the same elections.
    ///
    /// The OSV4-3 format normally includes an UTF-8 BOM and does not include
    /// a final newline. To achieve this behavior, the `include_bom` parameter
    /// should be set to `true`, whereas the `include_final_newline` parameter
    /// should be set to `false`.
    pub fn as_osv4_3_csv(
        &self,
        name_resolver: &impl NameResolver,
        include_bom: bool,
        include_final_newline: bool,
    ) -> Result<String, EMLError> {
        // Election metadata
        let count_contest = self
            .count
            .election
            .contests
            .first()
            .ok_or_else(|| EMLErrorKind::MissingContest.without_span())?;
        let election_identifier = &self.count.election.identifier;
        let authority_identifier = &self.managing_authority.authority_identifier;
        let election_id = election_identifier.id.cloned_value()?;
        let election_name = election_identifier.name.as_deref().unwrap_or("");
        let election_date = election_identifier.election_date.copied_value()?;
        let authority_name = authority_identifier.name.as_deref().unwrap_or("");
        let authority_id = authority_identifier.id.cloned_value()?;
        let authority_type = if self.count_type == CountType::Municipal {
            if SPECIAL_MUNICIPALITIES.contains(&authority_name) {
                "Openbaar lichaam"
            } else {
                "Gemeente"
            }
        } else {
            ""
        };

        info!("Generating OSV4-3 CSV for election {:?}", election_id);

        // Extract polling station information from the count contest, including cleaned names and postal codes
        let stations = utils::extract_polling_stations(self.count_type, count_contest)?;

        // Get the total votes for the contest (required for CSV output)
        let totals = count_contest
            .total_votes
            .as_ref()
            .ok_or_else(|| EMLErrorKind::MissingTotalVotes.without_span())?;

        // Build CSV output
        let mut output = CsvWriter::new(include_bom);

        // Rows 1-4: Header block
        output.row(["Verkiezing", "", election_name]);
        output.row(["Datum", "", &election_date.to_raw_value()]);
        output.row([
            "Gebied",
            "",
            format!("{} {}", authority_type, authority_name).trim(),
        ]);
        output.row(["Nummer", "", authority_id.value()]);

        // Row 5: empty separator row
        output.empty_row();

        // Row 6: column header, including polling station headers (but only for
        // 510b municipal count)
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

        // Row 7: Plain polling station numbers (i.e. without the HSB/CSB
        // prefix). For some reason this row is included even in the
        // non-municipal count CSVs, even though there are no polling stations
        // emitted in those CSVs.
        output.row(
            ["Gebiednummer", "", "", "", ""]
                .into_iter()
                .chain(stations.iter().map(|s| s.plain_ps_id.as_str())),
        );

        // Row 8: Postal codes of the polling stations
        // This row is only included for municipal level counts
        if self.count_type == CountType::Municipal {
            output.row(
                ["Postcode", "", "", "", ""]
                    .into_iter()
                    .chain(stations.iter().map(|s| s.postal_code.as_str())),
            );
        }

        // We now emit a couple of overal statistic rows on the entire election
        emit_stat_rows(&mut output, totals, &stations)?;

        // We get a list of votes (i.e. total votes and a list of votes for each selection) for each affiliation and candidate
        let counts = extract_vote_counts(name_resolver, totals, &stations)?;

        // Rows for each affiliation and candidate, including votes for each polling station
        for (_, affiliation) in counts.into_iter() {
            emit_affiliation_rows(&mut output, affiliation);
        }

        Ok(output.into_string(include_final_newline))
    }
}

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

fn emit_stat_rows(
    output: &mut CsvWriter,
    totals: &TotalVotes,
    stations: &[utils::CsvPollingStation],
) -> Result<(), EMLError> {
    // Helper macro to reduce repetition for various totals rows
    macro_rules! stat_row {
        ($label:expr, $votes_type:ident, $filter_type:expr) => {
            stat_row_start(
                $label,
                totals
                    .$votes_type
                    .get(&$filter_type)
                    .map(|v| v.raw())
                    .unwrap_or(Cow::Borrowed("0")),
            )
            .chain(stations.iter().map(|s| {
                s.reporting_unit
                    .$votes_type
                    .get(&$filter_type)
                    .map(|v| v.raw())
                    .unwrap_or(Cow::Borrowed("0"))
            }))
        };
    }

    output.row(
        stat_row_start("opgeroepenen", totals.eligible_voter_count.raw()).chain(
            stations
                .iter()
                .map(|s| s.reporting_unit.eligible_voter_count.raw()),
        ),
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
        stat_row_start("geldige stembiljetten", totals.candidate_votes_count.raw()).chain(
            stations
                .iter()
                .map(|s| s.reporting_unit.candidate_votes_count.raw()),
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

    let counted_ballots = (totals.blank_votes()?.copied_value()?
        + totals.invalid_votes()?.copied_value()?
        + totals.candidate_votes_count.copied_value()?)
    .to_string();
    let reporting_unit_counted_ballots = stations
        .iter()
        .map(|s| -> Result<Cow<str>, _> {
            Ok(Cow::Owned(
                (s.reporting_unit.blank_votes()?.copied_value()?
                    + s.reporting_unit.invalid_votes()?.copied_value()?
                    + s.reporting_unit.candidate_votes_count.copied_value()?)
                .to_string(),
            ))
        })
        .collect::<Result<Vec<_>, EMLError>>()?;
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

    Ok(())
}

fn emit_affiliation_rows(output: &mut CsvWriter, affiliation: AffiliationWithVotes) {
    // affiliation row starts with id and name in first and second columns,
    // then two empty columns, then total votes and votes for each polling
    // station (i.e. reporting unit)
    output.row(
        vec![
            affiliation.id.to_string().into(),
            affiliation.name.into(),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            affiliation.total_votes.to_string().into(),
        ]
        .into_iter()
        .chain(
            affiliation
                .reporting_unit_votes
                .iter()
                .map(ToString::to_string)
                .map(Cow::Owned),
        ),
    );

    for (_, candidate) in affiliation.candidates {
        // Candidate rows start with two empty columns, then candidate id and
        // name columsn, then total votes and votes for each polling station
        // (i.e. reporting unit)
        output.row(
            vec![
                Cow::Borrowed(""),
                Cow::Borrowed(""),
                candidate.id.to_string().into(),
                candidate.name.into(),
                candidate.total_votes.to_string().into(),
            ]
            .into_iter()
            .chain(
                candidate
                    .reporting_unit_votes
                    .iter()
                    .map(ToString::to_string)
                    .map(Cow::Owned),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        documents::candidate_lists::CandidateLists,
        io::{EMLParsingMode, EMLRead as _},
    };

    use super::*;

    #[test]
    fn test_gr2022_groningen() {
        let cl = CandidateLists::parse_eml(
            include_str!("../../test-files/csv/Kandidatenlijsten_GR2022_Groningen.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        let count = ElectionCount::parse_eml(
            include_str!("../../test-files/csv/Telling_GR2022_Groningen.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        let result = count.as_osv4_3_csv(&cl, true, false).unwrap();
        assert_eq!(
            result,
            include_str!("../../test-files/csv/osv4-3_telling_gr2022_groningen.csv")
        );
    }

    #[test]
    fn test_gr2022_west_maas_en_waal() {
        let cl = CandidateLists::parse_eml(
            include_str!("../../test-files/csv/Kandidatenlijsten_GR2022_WestMaasenWaal.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        let count = ElectionCount::parse_eml(
            include_str!("../../test-files/csv/Telling_GR2022_WestMaasenWaal.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        let result = count.as_osv4_3_csv(&cl, true, false).unwrap();
        assert_eq!(
            result,
            include_str!("../../test-files/csv/osv4-3_telling_gr2022_westmaasenwaal.csv")
        );
    }

    #[test]
    fn test_tk2025_west_maas_en_waal() {
        let cl = CandidateLists::parse_eml(
            include_str!("../../test-files/csv/Kandidatenlijsten_TK2025_Nijmegen.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        let count = ElectionCount::parse_eml(
            include_str!("../../test-files/csv/Telling_TK2025_gemeente_West_Maas_en_Waal.eml.xml"),
            EMLParsingMode::Strict,
        )
        .unwrap();

        let result = count.as_osv4_3_csv(&cl, true, false).unwrap();
        assert_eq!(
            result,
            include_str!("../../test-files/csv/osv4-3_telling_tk2025_gemeente_westmaasenwaal.csv")
        );
    }
}

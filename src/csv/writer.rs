use tracing::debug;

/// Quote a single CSV field: always wrapped in `"` (unless field is empty), internal `"` doubled.
fn qf(s: &str) -> String {
    if s.is_empty() {
        return "".to_string();
    }
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// A simple CSV writer that produces output consistent with existing OSV4-3 CSV files
///
/// This is a bit of a misnomer, as the CSV format as defined uses semicolons for
/// separation.
pub struct CsvWriter {
    output_str: String,
}

impl CsvWriter {
    /// Create a new Output with an optional UTF-8 BOM at the beginning.
    pub fn new(include_bom: bool) -> Self {
        let mut output_str = String::new();
        if include_bom {
            debug!("Including UTF-8 BOM in output");
            output_str.push('\u{FEFF}'); // UTF-8 BOM
        }
        Self { output_str }
    }

    /// Add a row to the CSV output, provided an iterator for field contents
    pub fn row(&mut self, fields: impl IntoIterator<Item = impl AsRef<str>>) {
        let fields_data = fields
            .into_iter()
            .map(|f| qf(f.as_ref()))
            .collect::<Vec<_>>();
        debug!("Emitting row with {} fields", fields_data.len());
        let row = fields_data.join(";");
        self.output_str.push_str(&row);
        self.output_str.push('\n');
    }

    /// Add an empty row to the CSV output
    pub fn empty_row(&mut self) {
        debug!("Adding empty row");
        self.output_str.push('\n');
    }

    /// Finalize the CSV output and return it as a string.
    pub fn into_string(mut self, trailing_newline: bool) -> String {
        if !trailing_newline && self.output_str.ends_with('\n') {
            debug!("Removing trailing newline");
            self.output_str.pop();
        } else if trailing_newline && !self.output_str.ends_with('\n') {
            debug!("Adding trailing newline");
            self.output_str.push('\n');
        }

        self.output_str
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_csv_writer() {
        let mut writer = super::CsvWriter::new(true);
        // field with a quote in it, should be escaped
        writer.row(["field1", "field;2", "field\"3"]);
        writer.empty_row();
        // empty field, should result in two separators with nothing in between
        writer.row(["field4", "", "field5"]);
        let output = writer.into_string(false);
        assert_eq!(
            output,
            concat!(
                "\u{feff}",
                r#""field1";"field;2";"field""3""#,
                "\n\n",
                r#""field4";;"field5""#
            )
        );
    }

    #[test]
    fn test_csv_writer_no_bom() {
        let mut writer = super::CsvWriter::new(false);
        writer.row(["field1", "field;2", "field\"3"]);
        let output = writer.into_string(false);
        assert_eq!(output, r#""field1";"field;2";"field""3""#);
    }

    #[test]
    fn test_csv_writer_trailing_newline() {
        let mut writer = super::CsvWriter::new(false);
        writer.row(["field1", "field;2", "field\"3"]);
        let output = writer.into_string(true);
        assert_eq!(output, concat!(r#""field1";"field;2";"field""3""#, "\n"));
    }
}

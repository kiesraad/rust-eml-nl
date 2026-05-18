edit: test for the CLA bot

# Kiesraad EML_NL Reader/Writer
This is a Rust crate that parses/emits EML_NL XML documents. Within this
repository we sometimes interchangeably use EML and EML_NL, but the software
only supports the EML_NL specification.

Multiple versions of the EML_NL specification exist. For more details on the
specification you can visit the [Kiesraad website][1]. For the schema
definitions you can also visit the [EML_NL repository][2] on GitHub. The version
of EML_NL that the current code supports is always linked as a git submodule
on this repository as well. This current branch is aiming to support EML_NL
versions 1.2.1 and 1.2.2.

This library requires the entire EML_NL file to be available in memory at once
and does not support streaming while reading or writing the document. The
memory overhead of a parsed document is relatively low however. It is able to
parse the entire GR2022 election definitions, candidate lists, counts and
results of all municipalties that participated at that time in approximately
5 seconds (a little under 500MB of XML data) on a small laptop. The library
should be functional on virtually any supported Rust target (including most
desktop and mobile OSes and the web).

[1]: https://www.kiesraad.nl/verkiezingen/osv-en-eml/eml-standaard
[2]: https://github.com/kiesraad/EML_NL/

## Library usage
For information on how to use this library please reference the generated
rust docs. For the newest released version you can check these on [docs.rs][3].

[3]: https://docs.rs/eml-nl

## Existing EML_NL data
The Kiesraad releases EML_NL documents for each election. To find these, you can
visit [data.overheid.nl][4] and go to the Kiesraad organization page. Once you 
find the election you are interested in, you can download the EML_NL documents 
from the 'Databronnen' page (look for EML in the title of the data source).

[4]: https://data.overheid.nl/community/organization/kiesraad

## Utilities
Aside from the library this repository also includes some helpful tools that
use the library to offer separate functionality. These tools are described
below.

### EML Validator
The EML validator is a validator for EML_NL documents. It will output any
warnings and errors that are contained within a document. If you have a document
with known issues that don't get reported, or if you found an EML_NL file that
incorrectly emits a warning or error, please report an issue. The validator will
also emit the size of a document and its hash. You can also print an internal
representation of the document using the `--debug` flag and print an emitted
pretty printed document by using the `--print` flag. For more information about
the capabilities of the validator you can run `--help`.

### eml2csv
This is a direct conversion of the eml2csv tool. This tool can be used to
transform the combination of an EML-230b candidate list and EML-510b municipal
count document to generate a CSV file in 'osv4-3' format. The input of this tool
is a little more flexible, you may provide the required files in any order you
want. You also have a couple of options to customize the output format for when
you are not working with Excel (omitting the UTF-8 BOM and including a final
newline).

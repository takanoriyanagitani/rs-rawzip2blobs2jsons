use clap::Parser;
use rs_rawzip2blobs2jsons::stdin2zip2blobs2jsons2stdout;
use std::process;

const MAX_ZIP_BYTES_DEFAULT: u64 = 1 << 20;
const MAX_ITEM_BYTES_DEFAULT: u64 = 1 << 17; // 131072

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Processes a zip archive from stdin, extracting files as JSON blobs to stdout.",
    long_about = "This CLI tool reads a zip archive from standard input. Each file within the zip is transformed into a JSON object (a 'blob') containing its metadata and base64-encoded content. These JSON blobs are then written to standard output, one per line. The tool allows specifying maximum sizes for the entire zip and individual items, and defines default content type and encoding for the extracted items."
)]
struct Cli {
    #[arg(
        long,
        default_value_t = MAX_ZIP_BYTES_DEFAULT,
        help = "Maximum size in bytes for the entire input zip file. If the zip file exceeds this limit, only data up to the limit will be read, which may lead to parsing errors for a truncated archive."
    )]
    zip_size_max: u64,

    #[arg(
        long,
        default_value = "unknown.zip",
        help = "Logical name for the input zip file, used in the metadata of each output JSON blob."
    )]
    zip_name: String,

    #[arg(
        long,
        default_value_t = MAX_ITEM_BYTES_DEFAULT,
        help = "Maximum size in bytes for an individual item (file) within the zip archive. Items exceeding this size will be skipped and not included in the output."
    )]
    item_size_max: u64,

    #[arg(
        long,
        default_value = "application/octet-stream",
        help = "Default Content-Type for items within the zip archive, if not specified elsewhere."
    )]
    item_content_type: String,

    #[arg(
        long,
        default_value = "identity",
        help = "Default Content-Encoding for items within the zip archive, if not specified elsewhere (e.g., 'gzip', 'deflate')."
    )]
    item_content_encoding: String,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Enable verbose output, including warnings for skipped items."
    )]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = stdin2zip2blobs2jsons2stdout(
        &cli.zip_name,
        &cli.item_content_type,
        &cli.item_content_encoding,
        cli.zip_size_max,
        cli.item_size_max,
        cli.verbose,
    ) {
        eprintln!("failed to process zip from stdin: {e}");
        process::exit(1);
    }
}

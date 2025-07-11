use clap::Parser;
use rhisto::{ColumnParser, Histogram};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

fn main() {
    let args = Args::parse();

    let mut reader: Box<dyn BufRead> = match args.input {
        Some(path_buf) => Box::new(BufReader::new(
            File::open(&path_buf).expect("failed to open input file"),
        )),
        None => Box::new(BufReader::new(std::io::stdin())),
    };

    if args.skip_header {
        reader.skip_until(b'\n').expect("failed to skip header");
    }

    let values: Vec<f32> = ColumnParser::<f32, _>::new(reader, args.column, &args.delim)
        .rows()
        // Panic on IO errors.
        // Discard rows with no value.
        .filter_map(|row| row.unwrap())
        .collect();

    let histo = Histogram::from_values(values, args.num_bins);

    let mut writer: Box<dyn Write> = match args.output {
        Some(path_buf) => Box::new(BufWriter::new(
            File::create(&path_buf).expect("failed to open output file"),
        )),
        None => Box::new(BufWriter::new(std::io::stdout())),
    };

    let _ = writeln!(writer, "bin_label,bin_value");
    for bin in histo.into_bins().iter() {
        let _ = writeln!(writer, "{:0.2}{}{:0.2}", bin.label, &args.delim, bin.count);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The optional buffer to read data from.
    input: Option<PathBuf>,

    /// The optional path to write histogram data to.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// The zero indexed column in the input buffer to read.
    #[arg(short, long, default_value_t = 0)]
    column: usize,

    /// The delimeting pattern used to separate columns in the input.
    #[arg(short, long, default_value = ",")]
    delim: String,

    /// Indicate whether the input data contains a header row.
    #[arg(short, long, default_value_t = false)]
    skip_header: bool,

    /// The number of bins in the histogram.
    #[arg(short, long, default_value_t = 10)]
    num_bins: usize,
}

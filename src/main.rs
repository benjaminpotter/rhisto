use clap::Parser;
use rhisto::{get_bin_index, get_bin_label, get_data_at_column};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

fn main() {
    let args = Args::parse();

    let data: Vec<f32> = match args.input {
        Some(path_buf) => {
            let file = File::open(&path_buf).expect("failed to open input file");
            let mut reader = BufReader::new(file);

            if args.skip_header {
                reader.skip_until(b'\n').expect("failed to skip header");
            }

            // Read column from each line.
            reader
                .lines()
                .map(|line| match line {
                    Ok(buf) => get_data_at_column(&buf, args.column, &args.delim)
                        .expect("failed to parse row"),
                    Err(e) => {
                        println!("failed to read line: {}", e);
                        f32::NAN
                    }
                })
                .collect()
        }
        // TODO: Support reading from STDIN.
        None => unimplemented!(),
    };

    // Find the data interval.
    let (min, max): (f32, f32) = data
        .iter()
        .fold(None, |acc: Option<(f32, f32)>, &x| match acc {
            Some((min, max)) => Some((min.min(x), max.max(x))),
            None => Some((f32::INFINITY, f32::NEG_INFINITY)),
        })
        .expect("no data provided");

    let mut bins: Vec<u32> = vec![0; args.num_bins];
    for bin in data
        .iter()
        .map(|&x| get_bin_index(x, min, max, args.num_bins))
    {
        bins[bin] += 1;
    }

    match args.output {
        Some(_) => unimplemented!(),
        None => {
            println!("bin_label,bin_value");
            for (i, x) in bins.iter().enumerate() {
                println!(
                    "{:0.2}{}{:0.2}",
                    get_bin_label(i, min, max, args.num_bins),
                    &args.delim,
                    x
                );
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The optional buffer to read data from.
    input: Option<PathBuf>,

    /// The optional path to write histogram data to.
    output: Option<PathBuf>,

    /// The column in the input buffer to read.
    #[arg(short, long, default_value_t = 1)]
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

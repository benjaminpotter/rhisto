# rhisto
A command line utility for histogramification.

## Usage
```
Usage: rhisto [OPTIONS] [INPUT] [OUTPUT]

Arguments:
  [INPUT]   The optional buffer to read data from
  [OUTPUT]  The optional path to write histogram data to

Options:
  -c, --column <COLUMN>      The column in the input buffer to read [default: 1]
  -d, --delim <DELIM>        The delimeting pattern used to separate columns in the input [default: ,]
  -s, --skip-header          Indicate whether the input data contains a header row
  -n, --num-bins <NUM_BINS>  The number of bins in the histogram [default: 10]
  -h, --help                 Print help
  -V, --version              Print version
```

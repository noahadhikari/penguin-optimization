# penguin-project

- [penguin-project](#penguin-project)
- [Rust Instructions](#rust-instructions)
  - [Requirements](#requirements)
  - [Usage](#usage)
    - [`list` or `ls`](#list-or-ls)
    - [`solve`](#solve)
      - [EXAMPLES:](#examples)
  - [Directory Structure](#directory-structure)
  - [Development](#development)
  - [Documentation](#documentation)
- [Python Instructions](#python-instructions)
  - [Requirements](#requirements-1)
  - [Usage](#usage-1)
    - [Generating instances](#generating-instances)
    - [Solving](#solving)
    - [Merging](#merging)
    - [Visualizing Instances](#visualizing-instances)


# Rust Instructions

## Requirements

It is recommended to use Linux or [WSL](https://docs.microsoft.com/en-us/learn/modules/get-started-with-windows-subsystem-for-linux/) since we use [coin-or cbc](https://www.coin-or.org/Cbc/), which is easier to set up on Linux.

First, install Rust using `rustup` by following the [instructions on the website](https://www.rust-lang.org/tools/install
), by running
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Next, install `coin-or cbc`, the LP solver we currently use by either running the below (for Linux) or following the [instructions on their repo](https://github.com/coin-or/Cbc)
```bash
sudo apt-get install coinor-cbc coinor-libcbc-dev
```

You will need a C compiler:

```bash
sudo apt-get install gcc
```

If you want to access the api:

```bash
sudo apt-get install pkg-config libssl-dev
```


## Usage

This command builds and runs the project
```bash
cargo run --release -- <SUBCOMMAND>
```
OR equivalently,first build the project with
```bash
cargo build --release
```
Then, in the root of the directory (or ensuring the inputs folder is in the same directory), run
```bash
./target/release/penguin-project <SUBCOMMAND>
```
Or equivalently (this builds it for you)


### `list` or `ls`
This lists all available solvers

### `solve`
USAGE:
```bash
... solve -s <SOLVER> <PATHS>..
```

Where you can input any number of `PATH` arguments, each one in the form `<size>/<ids>`

- `<size>` can be `small`, `medium`, or `large`
- `<ids>` can be a single id or a range of ids

#### EXAMPLES:

`solve -s lp large` runs the `lp` solver on everything in the large folder

`solve -s greedy small/1..220 medium/1` runs the `greedy` solver on ids 001 through 220 in the small folder and id 001 in the medium


## Directory Structure



## Development

A github workflow runs rustfmt whenever pushing to main or creating a pull request to main but its a good idea to install and run:
```bash
rustup component add rustfmt
cargo fmt
```
On the Nightly toolchain (to support features on the `rustfmt.toml`):
```bash
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly
cargo +nightly fmt
```


## Documentation

In addition to the above, we used the following crates/libraries:
| | | | 
|-|-|-|
|`good_lp`| [Github](https://github.com/rust-or/good_lp) | [Documentation](https://docs.rs/good_lp/1.3.2/good_lp/) |
|`clap`| [Derive Doc](https://github.com/clap-rs/clap/blob/v3.1.12/examples/derive_ref/README.md) | [Derive Tutorial](https://github.com/clap-rs/clap/blob/v3.1.12/examples/tutorial_derive/README.md#validated-values) |
|`pfh` | [Documentation](https://docs.rs/phf/0.10.1/phf/) ||
|`rustfmt-check`| [Github](https://github.com/mbrobbel/rustfmt-check) | [Actions Marketplace](https://github.com/marketplace/actions/rust-rustfmt-check) |
|`rustfmt` | [Github](https://github.com/rust-lang/rustfmt) | [Toml Docs](https://rust-lang.github.io/rustfmt) |


#
# Python Instructions

## Requirements

A Python skeleton is available in the `python` subdirectory. The Python
skeleton was developed using Python 3.9, but it should work with Python
versions 3.6+.

## Usage

### Generating instances

To generate instances, read through [`python/instance.py`](python/instance.py),
which contains a dataclass (struct) that holds the data for an instance, as
well as other relevant methods. Then modify the
[`python/generate.py`](python/generate.py) file by filling in the
`make_{small,medium,large}_instance` functions.

After you have filled in those functions, you can run `make generate` in the
`python` directory to generate instances into the input directory.

To run unit tests, run `make check`.

### Solving

We've created a solver skeleton at [`python/solve.py`](python/solve.py).
```bash
python3 solve.py case.in --solver=naive case.out
```

We've also created a skeleton that runs your solver on all cases and puts them
in the output directory. To use it, modify
[`python/solve_all.py`](python/solve_all.py) to use your solver function(s).
Then run

```
python3 python/solve_all.py inputs outputs
```

in the root directory.


### Merging

To merge multiple output folders, taking the best solutions, see
[`python/merge.py`](python/merge.py).


### Visualizing Instances

To visualize problem instances, run `python3 visualize.py`, passing  in the
path to your `.in` file as the first argument (or `-` to read from standard
input). To visualize a solution as well, pass in a `.out` file to the option
`--with-solution`.

By default, the output visualization will be written as a SVG file to standard
output. To redirect it to a file, use your shell's output redirection or pass
in an output file as an additional argument.

For example, you could run
```bash
python3 visualize.py my_input.in out.svg
```
to create an `out.svg` file visualizing the `my_input.in` problem instance.

To visualize a solution file for this instance as well, you could run
```bash
python3 visualize.py my_input.in --with-solution my_soln.out out.svg
```

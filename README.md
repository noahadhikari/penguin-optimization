# penguin-project

- [penguin-project](#penguin-project)
- [Rust Instructions](#rust-instructions)
  - [Requirements](#requirements)
  - [Usage](#usage)
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

It is recommended to use Linux or [WSL](https://docs.microsoft.com/en-us/learn/modules/get-started-with-windows-subsystem-for-linux/) since we use [coin-or cbc](https://www.coin-or.org/Cbc/), which is easer to setup in linux.

First install rust by using `rustup` by following the [instructions on the website](https://www.rust-lang.org/tools/install
), by running
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Next install `coin-or cbc`, the LP solver we currently use by either running the below (for linux) or following the [instructions on their github](https://github.com/coin-or/Cbc)
```bash
sudo apt-get install coinor-cbc coinor-libcbc-dev
```

If (when) we move over to the `HiGHS` solver, you will need a C compiler.

## Usage

In the root of the directory, run
```bash
cargo run --release
```
TODO: create and define cli arguments


## Directory Structure



## Development

A github workflow runs rustfmt whenever pushing to main or creating a pull request to main but its a good idea to install and run:
```bash
rustup component add rustfmt
cargo fmt
```


## Documentation

In addition to the above, we used the following crates/libraries:
| | | | 
|-|-|-|
|`good_lp`| [Github](https://github.com/rust-or/good_lp) | [Documentation](https://docs.rs/good_lp/1.3.2/good_lp/) |
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

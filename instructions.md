# Spring 2022 CS170 Project Instructions
You must have Rust (and Cargo) installed to run the code.
One of our strategies relies on LP, which calls [Cbc](https://projects.coin-or.org/Cbc) to solve the problem.
This means you must have Cbc installed, which you can install following the instructions [here](https://github.com/coin-or/Cbc). (On Windows, use WSL for easier installation).

To install rustup:

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

To install CBC:

`sudo apt-get install coinor-cbc coinor-libcbc-dev`

If faced with `linker cc not found` error:

`sudo apt-get install gcc`

Once you have everything set up, you can run the code simply by running `cargo run --release` from the command line.

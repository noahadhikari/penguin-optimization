// Used to ignore unused code warnings.
#![allow(dead_code)]

// extern crates
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;

mod api;
mod grid;
mod lp;
mod point;
mod solvers;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use api::{get_api_result, InputType};
use clap::{Parser, Subcommand};
use grid::Grid;
use phf::phf_map;
use solvers::*;

// Define solver functions
type SolverFn = fn(&mut Grid, &str);

static SOLVERS: phf::Map<&'static str, SolverFn> = phf_map! {
	"benchmark" => benchmark_greedy,
	"greedy" => greedy,
	"rlp" => randomize_valid_solution_with_lp_threaded,
	"hillclimb" => hillclimb,
	"rand_hillclimb" => rand_hillclimb_threaded,
};


// Define command line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
	#[clap(subcommand)]
	command: Commands,
}


#[derive(Subcommand)]
enum Commands {
	/// List all solvers
	#[clap(alias = "ls")]
	List,

	/// Query the API
	#[clap(alias = "q")]
	Api {
		#[clap(default_value = "s", parse(try_from_str=api::input_size_from_string))]
		size: InputType,
	},

	/// Run a solver on several specified inputs
	#[clap(arg_required_else_help = true)]
	Solve {
		/// Solver to use
		#[clap(short, parse(try_from_str=get_solver))]
		solver: SolverFn,

		/// Inputs to the solver <size>/<id>
		///
		/// large/1..4 OR large OR large/1..4 small/5
		#[clap(required = true,	parse(try_from_str=get_paths))]
		paths: Vec<Vec<(PathBuf, PathBuf)>>,
		// Vec allows for multiple inputs in the after the solver name
	},
}


fn main() {
	let args = Args::parse();

	match &args.command {
		// -- LIST --
		Commands::List => {
			println!("List of solvers:");
			for (name, _) in SOLVERS.entries() {
				println!("\t{}", name);
			}
		}
		// -- API --
		Commands::Api { size } => {
			get_api_result(size);
		}
		// -- SOLVE --
		Commands::Solve { solver, paths } => {
			// Prevent solving multiple identical inputs
			let mut path_list: HashSet<&PathBuf> = HashSet::new();

			// TODO: Make this parallel
			// Run the solver on each input
			for path_set in paths {
				for (input, output) in path_set {
					// Ensure this input is unique
					if path_list.contains(&input) {
						continue;
					}
					path_list.insert(&input);
					println!("{}", input.display());


					// let mut grid = Grid::from_file(input);
					let mut grid = Grid::from_file(input.to_str().unwrap())
						.expect(format!("Failed to load grid from {}", input.to_str().unwrap()).as_str());

					solver(&mut grid, output.to_str().unwrap());
				}
			}
		}
	}
}

// -- Input parsing and validation --

fn check_id_range(id: u8) -> Result<bool, String> {
	const ID_RANGE: std::ops::RangeInclusive<u8> = 1..=239;

	if !ID_RANGE.contains(&id) {
		Err(format!(
			"Id must be an integer between {} and {}",
			ID_RANGE.start(),
			ID_RANGE.end()
		))
	} else {
		Ok(true)
	}
}

// Converts input string to a list of input and output paths
fn get_paths(input: &str) -> Result<Vec<(PathBuf, PathBuf)>, String> {
	// Assuming run from root directory
	let mut paths: Vec<(PathBuf, PathBuf)> = Vec::new();

	let size = input
		.split(std::path::MAIN_SEPARATOR)
		.next()
		.ok_or("Error parsing input")?;
	let id = input.split(std::path::MAIN_SEPARATOR).skip(1).next();

	let in_path = Path::new("./inputs").join(size);
	let out_path = Path::new("./outputs").join(size);

	match id {
		// Return a subset of the files in the directory
		Some(id) => {
			let mut id_range = id.split("..");
			let id_start = id_range
				.next()
				.ok_or("Error parsing input")?
				.parse::<u8>()
				.map_err(|_| "id must be an integer")?;
			let id_end = id_range.next();

			check_id_range(id_start)?;

			match id_end {
				Some(id_end) => {
					// Given a range
					let id_end = id_end.parse::<u8>().map_err(|_| "id must be an integer")?;

					check_id_range(id_end)?;
					// Check that id start <= id end
					(id_start <= id_end)
						.then(|| 1)
						.ok_or("start id must be less than end id")?;

					for i in id_start..=id_end {
						let mut current_in = in_path.clone();
						let mut current_out = out_path.clone();

						// https://stackoverflow.com/questions/50458144/
						current_in.push(format!("{:0>3}", i));
						current_in.set_extension("in");
						current_out.push(format!("{:0>3}", i));
						current_out.set_extension("out");

						paths.push((current_in, current_out));
					}
				}
				None => {
					// Given a single id
					let mut current_in = in_path.clone();
					let mut current_out = out_path.clone();

					current_in.push(format!("{:0>3}", id));
					current_in.set_extension("in");
					current_out.push(format!("{:0>3}", id));
					current_out.set_extension("out");

					paths.push((current_in, current_out));
				}
			}
			// Return the created vector
			Ok(paths)
		}
		None => {
			// Return all files the directory
			let dir = fs::read_dir(in_path).map_err(|_| "Error reading directory")?;

			for path in dir {
				let path = path.map_err(|_| "Error reading directory")?.path();

				// path will be in the form of "inputs/size/id.in"
				let mut current_out = out_path.clone().join(path.file_stem().unwrap());

				current_out.set_extension("out");

				paths.push((path, current_out));
			}

			Ok(paths)
		}
	}
}

// Validates and converts a string to a solver function
fn get_solver(solver: &str) -> Result<SolverFn, String> {
	SOLVERS
		.get(solver)
		.cloned()
		.ok_or("Solver not found, run list to see possible solvers".to_string())
}

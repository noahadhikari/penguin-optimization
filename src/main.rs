// Used to ignore unused code warnings.
#![allow(dead_code)]

mod grid;
mod lp;
mod point;
mod solvers;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use grid::Grid;
use phf::phf_map;
use solvers::*;
use stopwatch::Stopwatch;

// Define solver functions

type SolverFn = fn(&Grid, &mut Grid);

static SOLVERS: phf::Map<&'static str, SolverFn> = phf_map! {
	"greedy" => benchmark_greedy,
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

	/// Run a solver on several specified inputs
	#[clap(arg_required_else_help = true)]
	Solve {
		/// Solver to use
		#[clap(short, parse(try_from_str=get_solver))]
		solver: SolverFn,

		/// Inputs to the solver <size>/<id>
		///
		/// large/1..4 OR large OR large/1..4 small/5
		#[clap(required = true,	parse(try_from_str=get_input_paths))]
		paths: Vec<Vec<PathBuf>>,
		// Vec allows for multiple inputs in the after the solver name
	},
}

fn main() {
	let args = Args::parse();

	match &args.command {
		// -- LIST --
		Commands::List => {
			println!("List of solvers:");
		}
		// -- SOLVE --
		Commands::Solve { solver, paths } => {
			// Collapse the multiple paths given into one set
			let path_list: HashSet<&PathBuf> = HashSet::from_iter(paths.iter().map(|vec| vec.iter()).flatten());
			// TODO: Maintain input order and use a different method to prevent multiple
			// inputs from the same file

			// TODO: Make this parallel
			// Run the solver on each input
			for path in path_list {
				// let grid = Grid::from_file(path);
				println!("{:?}", path);
				let grid = get_grid(path.to_str().unwrap())
					.expect(format!("Failed to load grid from {}", path.to_str().unwrap()).as_str());
				let mut sol = grid.clone();

				solver(&grid, &mut sol);
				// println!("{:#}", sol);
				// println!("{:#}", grid);
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

// Converts input string to a list of paths
fn get_input_paths(input: &str) -> Result<Vec<PathBuf>, String> {
	// Assuming run from root directory
	let mut inputs: Vec<PathBuf> = Vec::new();

	let size = input
		.split(std::path::MAIN_SEPARATOR)
		.next()
		.ok_or("Error parsing input")?;
	let id = input.split(std::path::MAIN_SEPARATOR).skip(1).next();

	let path = Path::new("./inputs").join(size);

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
						let mut current = path.clone();

						// https://stackoverflow.com/questions/50458144/
						current.push(format!("{:0>3}", i));
						current.set_extension("in");

						inputs.push(current);
					}
				}
				None => {
					// Given a single id
					let mut current = path.clone();

					current.push(format!("{:0>3}", id));
					current.set_extension("in");

					inputs.push(current);
				}
			}
			// Return the created vector
			Ok(inputs)
		}
		None => {
			// Return all files the directory
			Ok(
				fs::read_dir(path)
					.map_err(|_| "Error reading directory")?
					.map(|entry| entry.unwrap().path())
					.collect::<Vec<PathBuf>>(),
			)
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

// -- --

fn solve_all_inputs() {
	const CUTOFF_TIME: u32 = 500000; // max time in seconds

	let paths = fs::read_dir("./inputs/small").unwrap();

	for path in paths {
		let real_path = path.unwrap().path();
		// ie: 001
		let test_number = real_path.file_stem().unwrap().to_str().unwrap();
		let input_path = real_path.to_str().unwrap();
		let output_path = "./outputs/".to_string() + "small/" + test_number + ".out";

		let mut grid = get_grid(input_path).unwrap();

		grid.lp_solve(CUTOFF_TIME);

		write_sol(&grid, &output_path);
	}
}

fn solve_one_input() {
	const INPUT_PATH: &str = "./inputs/test/tiny.in";
	const OUTPUT_PATH: &str = "./outputs/test/tiny.out";
	let mut grid = get_grid(INPUT_PATH).unwrap();
	const CUTOFF_TIME: u32 = 3600; // max time in seconds
	grid.lp_solve(CUTOFF_TIME);

	write_sol(&grid, OUTPUT_PATH);
	// println!("Valid: {}", grid.is_valid());
	println!("{}", grid);
}

fn solve_all_randomized() {
	let paths = fs::read_dir("./inputs/medium").unwrap();

	// Will find a better way for this
	let mut i = 1;
	for path in paths {
		// There's probably a much better way to do this
		match i {
			1..=10 => {
				// Uncomment for 11-20
				// i += 1;
				// continue
			}
			11..=20 => {
				// Comment out for 11-20
				continue;
			}
			_ => return,
		}

		let real_path = path.unwrap().path();
		let test_number = real_path.file_stem().unwrap().to_str().unwrap(); // ie: 001
		let input_path = real_path.to_str().unwrap();
		let output_path = "./outputs/".to_string() + "medium/" + test_number + ".out";
		solve_one_randomized(input_path, &output_path, 10);

		i += 1;
	}
}

fn solve_one_randomized(input_path: &str, output_path: &str, secs_per_input: u64) {
	// const INPUT_PATH: &str = "./inputs/medium/001.in";
	// const OUTPUT_PATH: &str = "./outputs/medium/001.out";
	const CUTOFF_TIME: u32 = 60; // max time in seconds
	const ITERATIONS: u32 = 10000;

	let mut best_penalty_so_far = f64::INFINITY;
	let mut best_grid_so_far = Grid::new(0, 0, 0);
	let sw = Stopwatch::start_new();
	// For every file:
	while sw.elapsed().as_secs() < secs_per_input {
		// 5 mins
		let mut grid = get_grid(input_path).unwrap(); // Need a way to move this out
		let p = grid.random_lp_solve(CUTOFF_TIME);
		// println!("{} penalty: {}", i, p);
		if p < best_penalty_so_far {
			best_penalty_so_far = best_penalty_so_far.min(p);
			best_grid_so_far = grid;
		}

		let time = sw.elapsed().as_secs();
		if sw.elapsed().as_secs() % 10 == 0 {
			println!("{} secs passed. Best so far: {}", time, best_penalty_so_far);
		}
	}
	println!("Best: {}", best_penalty_so_far);
	println!("Valid: {}", best_grid_so_far.is_valid());
	write_sol(&best_grid_so_far, output_path);
}


// Algorithms

/// Returns the grid created from the passed in input file.
fn get_grid(path: &str) -> io::Result<Grid> {
	let mut g = Grid::new(0, 0, 0);

	let file = File::open(path)?;
	let reader = BufReader::new(file);

	let mut i: i32 = 0;
	let mut num_cities: i32 = -1;
	for line in reader.lines() {
		if let Ok(l) = line {
			let vec: Vec<&str> = l.split_whitespace().collect();
			let first_val: &str = vec.get(0).unwrap();
			if first_val.eq("#") {
				continue;
			}
			match i {
				0 => num_cities = first_val.parse::<i32>().unwrap(),
				1 => g.set_dimension(first_val.parse::<u8>().unwrap()),
				2 => g.set_service_radius(first_val.parse::<u8>().unwrap()),
				3 => g.set_penalty_radius(first_val.parse::<u8>().unwrap()),
				_ => {
					if (4..(4 + num_cities)).contains(&i) {
						let x = first_val.parse::<i32>().unwrap();
						let y = vec.get(1).unwrap().parse::<i32>().unwrap();
						g.add_city(x, y);
					}
				}
			}
			i += 1;
		}
	}
	Ok(g)
}

fn write_sol(grid: &Grid, path: &str) {
	// Only overwrite if solution is better than what we currently have
	if Path::new(path).is_file() {
		let file = File::open(path).unwrap();
		let reader = BufReader::new(file);
		let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
		let penalty_line = lines.get(0).unwrap(); // Penalty = xxx
		let split_line: Vec<&str> = penalty_line.split_whitespace().collect();
		let existing_penalty: f64 = split_line.get(3).unwrap().parse::<f64>().unwrap();

		if grid.penalty() >= existing_penalty {
			return;
		}
	}

	let data = grid.output();
	let mut f = OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(path)
		.expect("Unable to open file");
	f.write_all(data.as_bytes()).expect("Unable to write data");
}

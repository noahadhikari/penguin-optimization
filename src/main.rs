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
use std::collections::{HashSet, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use api::{get_penalty_from_file, get_api_result, InputType};
use clap::{Parser, Subcommand};
use grid::Grid;
use phf::phf_map;
use solvers::*;
use rayon::prelude::*;
use stopwatch::Stopwatch;

// Define solver functions

type SolverFn = fn(&mut Grid);

static SOLVERS: phf::Map<&'static str, SolverFn> = phf_map! {
	"benchmark" => benchmark_greedy,
	"greedy" => greedy,
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
					let mut grid = get_grid(input.to_str().unwrap())
						.expect(format!("Failed to load grid from {}", input.to_str().unwrap()).as_str());
					
					
					solver(&mut grid);
					// grid.to_file(output);
					write_sol(&grid, output.to_str().unwrap());
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
			let dir = fs::read_dir(in_path)
				.map_err(|_| "Error reading directory")?;
	
			for path in dir {
				let path = path
					.map_err(|_| "Error reading directory")?
					.path();

				// path will be in the form of "inputs/size/id.in"
				let mut current_out = out_path
					.clone()
					.join(path.file_stem().unwrap());

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
	let paths = fs::read_dir("./inputs/small").unwrap();

	for path in paths {
		let real_path = path.unwrap().path();
		let test_number = real_path.file_stem().unwrap().to_str().unwrap(); // ie: 001
		let input_path = real_path.to_str().unwrap();
		let output_path = "./outputs/".to_string() + "small/" + test_number + ".out";
		// solve_one_randomized(&get_grid(input_path).unwrap(), &output_path, 60);
		solve_one_random_threaded(&input_path, &output_path, 60);
	}
}

fn solve_one_randomized(grid_orig: &Grid, output_path: &str, secs_per_input: u64) {
	// const INPUT_PATH: &str = "./inputs/medium/001.in";
	// const OUTPUT_PATH: &str = "./outputs/medium/001.out";
	const CUTOFF_TIME: u32 = 60; // max time in seconds
	const ITERATIONS: u32 = 10000;

	let mut grid = (*grid_orig).clone();
	use rand::{thread_rng, Rng};
	let mut rng = thread_rng();
	let mut best_penalty_so_far = f64::INFINITY;
	// let mut grid = get_grid(input_path).unwrap();
	let mut best_towers_so_far = HashMap::new();
	let sw = Stopwatch::start_new();
	// For every file:

	// let mut i = 0;
	while sw.elapsed().as_secs() < secs_per_input {
		let p = grid.random_lp_solve(CUTOFF_TIME, rng.gen_range(1..=u32::MAX));
		// println!("{} penalty: {}", i, p);
		if p < best_penalty_so_far {
			best_penalty_so_far = p;
			best_towers_so_far = grid.get_towers_ref().clone();
			write_sol(&grid, output_path);
		}

		let time = sw.elapsed().as_secs();
		if sw.elapsed().as_secs() % 10 == 0 {
			println!("{} secs passed. Best so far: {}", time, best_penalty_so_far);
		}
		// i += 1;
	}
	println!("Best: {}", best_penalty_so_far);
	println!("Valid: {}", grid.is_valid());
	grid.replace_all_towers(best_towers_so_far);
}

fn solve_one_random_threaded(input_path: &str, output_path: &str, secs_per_input: u64) {
	let mut grids = vec![];
	for _ in 0..(num_cpus::get()) {
		let grid = get_grid(input_path).unwrap();
		grids.push(grid);
	}
	grids
		.par_iter()
		.for_each(|g| solve_one_randomized(g, output_path, secs_per_input));
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
		let existing_penalty = get_penalty_from_file(path);

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

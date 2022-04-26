// Used to ignore unused code warnings.
#![allow(dead_code)]

// extern crates
#[macro_use]
extern crate lazy_static;


mod grid;
mod lp;
mod point;

// crate imports
// use point::preprocess::setup_persistence;

// other imports
use std::collections::HashMap;
// use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufReader, Write};
use std::path::Path;
use std::{fs, u32};

use grid::Grid;
use rand::{thread_rng, Rng};
use stopwatch::Stopwatch;
use reqwest;
use serde::{Deserialize, Serialize};

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

	// Will find a better way for this
	let mut i = 1;
	for path in paths {
		// There's probably a much better way to do this
		// match i {
		// 	1..=10 => {
		// 		// Uncomment for 11-20
		// 		// i += 1;
		// 		// continue
		// 	}
		// 	11..=20 => {
		// 		// Comment out for 11-20
		// 		continue;
		// 	}
		// 	_ => return,
		// }

		let real_path = path.unwrap().path();
		let test_number = real_path.file_stem().unwrap().to_str().unwrap(); // ie: 001
		let input_path = real_path.to_str().unwrap();
		let output_path = "./outputs/".to_string() + "small/" + test_number + ".out";
		solve_one_randomized(input_path, &output_path, 60);

		i += 1;
	}
}

fn solve_one_randomized(input_path: &str, output_path: &str, secs_per_input: u64) {
	// const INPUT_PATH: &str = "./inputs/medium/001.in";
	// const OUTPUT_PATH: &str = "./outputs/medium/001.out";
	const CUTOFF_TIME: u32 = 60; // max time in seconds
	const ITERATIONS: u32 = 10000;

	use std::u32::MAX;

	use rand::{thread_rng, Rng};
	let mut rng = thread_rng();
	let mut best_penalty_so_far = f64::INFINITY;
	let mut grid = get_grid(input_path).unwrap();
	let mut best_towers_so_far = HashMap::new();
	let sw = Stopwatch::start_new();
	// For every file:

	let mut i = 0;
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
		i += 1;
	}
	println!("Best: {}", best_penalty_so_far);
	println!("Valid: {}", grid.is_valid());
	grid.replace_all_towers(best_towers_so_far);
}

fn main() {
	// solve_all_inputs();
	// solve_one_input();
	// solve_one_randomized("inputs/small/003.in", "outputs/small/003.out", 10);
	// setup_persistence();
	// solve_all_randomized();
	get_small_result();
}

// Algorithms

/// Greedy algorithm for benchmarking.
/// Places towers at all city locations that haven't been covered
fn place_at_cities(grid: &mut Grid) {
	let cities = grid.get_cities_ref().clone();
	let city_points = cities.keys();
	println!("{:?}", city_points);
	for point in city_points {
		let covered = grid.get_cities_ref().get(point).unwrap();
		if covered.len() > 0 {
			continue;
		}
		grid.add_tower(point.get_x(), point.get_y());
	}
}

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

fn get_penalty_from_file(path: &str) -> f64 {
	let file = File::open(path).unwrap();
	let reader = BufReader::new(file);
	let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
	let penalty_line = lines.get(0).unwrap(); // Penalty = xxx
	let split_line: Vec<&str> = penalty_line.split_whitespace().collect();
	let existing_penalty: f64 = split_line.get(3).unwrap().parse::<f64>().unwrap();
	existing_penalty
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


#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
	Entries: Vec<Scores>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Scores {
	TeamName: String,
	TeamScore: f64,
}

#[tokio::main]
pub async fn get_small_result() {
	let small_count: u8 = 241;
	for i in 235..=small_count {
		// let our_path  = "./outputs/small/".to_string() + &i.to_string() + ".in";
		let highest_score = get_highest_leaderboard_score(i).await;
		match highest_score {
			Ok(score) => {
				println!("{:?}", score);
			},
			Err(e) => panic!("{}", e),
		}
	}

		// if Path::new(&our_path).is_file() {
			// return Err("File not found".to_string());
		// }
		// let our_penalty = get_penalty_from_file(our_path.as_str());

		// let get_url = "https://project.cs170.dev/scoreboard/small/".to_string() + &i.to_string();
		// let res = reqwest::get(get_url)
		// 	.await
		// 	.unwrap();
			
		// match res.status() {
		// 	reqwest::StatusCode::OK => {
		// 		match res.json::<APIResponse>().await {
		// 			Ok(parsed) => {
		// 				// println!("Success! {:?}", parsed);
		// 				let x = parsed.Entries.get(0).unwrap().TeamScore;
		// 				println!("{:?}", x);
		// 			}
		// 			Err(_) => println!("Hm, the response didn't match the shape we expected."),
		// 		};
		// 	}
		// 	other => {
    //     panic!("Uh oh! Something unexpected happened: {:?}", other);
		// 	}
		// }
		
		
		// println!("{:?}", result)
		// println!("{}", get_url);
		// let client = reqwest::Client::new();
		// let response = client
		// 		.get(get_url)
		// 		.send()
		// 		.await
		// 		.unwrap();
		// println!("Success! {:?}", response);

		// GET their response from get_url
		// let body = reqwest::get("https://www.rust-lang.org")
    // .await?
    // .text()
    // .await?;

		
		// Maybe use serde_json to parse: https://stackoverflow.com/questions/30292752/how-do-i-parse-a-json-file	
		// sort by TeamScore and compare our_penalty with it. If it's less than, add info to a vector. 
	// }	
	// Ok(()) // Return the vector
}

/// Only for small for now
// #[tokio::main]
pub async fn get_highest_leaderboard_score(test_num: u8) -> Result<f64, String>{
	let get_url: String = "https://project.cs170.dev/scoreboard/small/".to_string() + &test_num.to_string();

	let res = reqwest::get(get_url)
			.await
			.unwrap();
			
		match res.status() {
			reqwest::StatusCode::OK => {
				match res.json::<APIResponse>().await {
					Ok(parsed) => return Ok(parsed.Entries.get(0).unwrap().TeamScore),
					Err(_) => return Err("The response didn't match the shape we expected.".to_string()),
				};
			}
			other => return Err("Other error occurred".to_string() + other.as_str()),
		}
}
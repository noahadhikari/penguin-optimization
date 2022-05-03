use std::collections::{HashMap, HashSet};

use colored::Colorize;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use stopwatch::Stopwatch;

use crate::annealing;
use crate::grid::Grid;
use crate::point::Point;


// Greedy parameters
// What percent of the total do we consider in addition to max coverage
const PERCENT_REMAINING: f32 = 0.25;

// LP parameters
// Max time in seconds
const LP_CUTOFF_TIME: u32 = 500000;

// RLP parameters
const SECS_PER_INPUT: u64 = 60;
const CUTOFF_TIME: u32 = 60; // max time in seconds
const ITERATIONS: u32 = 10000;

// Randomized hillclimb parameters

// How many iterations of hillclimb to do. When =0 then is threaded naive
// hillclimb.
const HILLCLIMB_ITERATIONS_PER_THREAD: usize = 0;
// Radius of hillclimb. works best with 3 (any), 8 (small), 10 (medium), 14
// (large). brute-force is grid dimension * sqrt 2: 43 (small), 71 (medium), 142
// (large)
const HILLCLIMB_RADIUS: u8 = 10;

// Simulated annealing parameters
const SA_ITERATIONS: u32 = 1000;
const SA_RADIUS: u8 = 43;

// ------- Solver functions -------

// -- Naive Greedy --
/// Greedy algorithm for benchmarking.
/// Places towers at all city locations that haven't been covered
pub fn benchmark_greedy(grid: &mut Grid, output_path: &str) {
	let cities = grid.get_cities_ref().clone();
	let city_points = cities.keys();

	for city in city_points {
		let covered = grid.get_cities_ref().get(city).unwrap();
		if covered.len() > 0 {
			continue;
		}
		grid.add_tower(city.get_x(), city.get_y());
	}
	grid.write_solution(output_path);
}

// -- Greedy --
/// Greedy algorithm for solving the grid.
/// Places a tower such that it covers the most cities.
/// Picks a range of covered and minimizes the added penalty.
pub fn greedy(grid: &mut Grid, output_path: &str) {
	let mut cities = grid.get_cities_ref().clone().into_keys().collect::<Vec<Point>>();

	// Continue until cities are covered
	while cities.len() != 0 {
		let mut d: HashMap<Point, u32> = HashMap::new();

		for city in &cities {
			for possible_tower in Point::points_within_radius(*city, grid.service_radius(), grid.dimension()).unwrap() {
				let counter = d.entry(*possible_tower).or_insert(0);
				*counter += 1
			}
		}

		// Towers to be considered, mapped to added cost
		let mut towers_to_be_considered: HashMap<Point, f64> = HashMap::new();

		// Grab among (us) the towers that cover the most
		let mut ordered_possibles: Vec<(Point, u32)> = d.into_iter().collect::<Vec<(Point, u32)>>();
		ordered_possibles.sort_by_key(|a| a.1);
		ordered_possibles.reverse();

		let max = ordered_possibles[0].1;
		let total = ordered_possibles.len();
		let mut index = 0;

		// First extract all max value ones
		for i in 0..total {
			let possible = ordered_possibles[i];
			if possible.1 == max {
				index += 1;
				towers_to_be_considered.insert(possible.0, 0.0);
			} else {
				break;
			}
		}

		// Next extract PERCENT_REMAINING of the rest
		let end = std::cmp::min(((total - index) as f32 * PERCENT_REMAINING) as usize, total);

		for i in index..end {
			towers_to_be_considered.insert(ordered_possibles[i].0, 0.0);
		}

		// Now test inserting each tower into grid, updating added cost value

		for (tower, cost) in towers_to_be_considered.iter_mut() {
			grid.add_tower(tower.get_x(), tower.get_y());
			*cost += grid.penalty();
			grid.remove_tower(tower.get_x(), tower.get_y())
		}

		// Pick the tower that adds the lowest cost
		let tower_to_add = towers_to_be_considered
			.iter()
			// Rust doesn't allow ordering by float, so just compare the integers for now
			// TODO https://docs.rs/float-cmp/0.5.2/float_cmp/ ?
			.max_by(|a, b| (*a.1 as u32).cmp(&(*b.1 as u32)))
			.unwrap()
			.0;

		grid.add_tower(tower_to_add.get_x(), tower_to_add.get_y());


		// Only consider cities not already covered

		let mut new_cities: Vec<Point> = Vec::new();
		for city in cities.iter() {
			if grid.get_cities_ref().get(city).unwrap().len() == 0 {
				new_cities.push(city.clone());
			}
		}
		cities = new_cities;
	}

	grid.write_solution(output_path);
}


// -- Linear Programming --
// TODO: move out of grid class
pub fn linear_programming(grid: &mut Grid) {
	grid.lp_solve(LP_CUTOFF_TIME);
}


// -- Randomize Valid Solution threaded
pub fn randomize_valid_solution_with_lp_threaded(grid: &mut Grid, output_path: &str) {
	let mut grids: Vec<_> = vec![];
	for _ in 0..(num_cpus::get()) {
		grids.push(grid.clone());
	}
	grids
		.par_iter_mut()
		.for_each(|g: &mut Grid| randomize_valid_solution_with_lp(g, output_path));
}


// -- Randomize Valid Solution with LP --
pub fn randomize_valid_solution_with_lp(grid: &mut Grid, output_path: &str) {
	let mut rng = thread_rng();
	let mut best_penalty_so_far = f64::INFINITY;
	let sw = Stopwatch::start_new();

	// Grab a valid solution and see if it is better
	// TODO: prevent getting same one over and over
	while sw.elapsed().as_secs() < SECS_PER_INPUT {
		let p = grid.random_lp_solve(CUTOFF_TIME, rng.gen_range(1..=u32::MAX));
		// println!("{} penalty: {}", i, p);
		if p < best_penalty_so_far {
			best_penalty_so_far = p;
			grid.write_solution(output_path);
		}

		let time = sw.elapsed().as_secs();
		if sw.elapsed().as_secs() % 10 == 0 {
			println!("{} secs passed. Best so far: {}", time, best_penalty_so_far);
		}
		// Reset grid
		grid.remove_all_towers();
	}
	println!("Best: {}", best_penalty_so_far);
}

/// First grabs the current solution we have.
/// Then, sees if any improvements can be made by moving a tower slightly, and
/// makes them.
pub fn hillclimb(grid: &mut Grid, output_path: &str) {
	// println!("Hillclimbing for {}", output_path);
	let initial_towers = Grid::towers_from_file(output_path);
	for tower in initial_towers {
		grid.add_tower(tower.x, tower.y);
	}
	let old_penalty = grid.penalty();

	if hillclimb_helper(grid, output_path, old_penalty) {
		grid.remove_all_towers();
		hillclimb(grid, output_path);
	}
	let new_penalty = grid.penalty();
	if new_penalty < old_penalty {
		println!("Improved! {} -> {}", old_penalty, new_penalty);
	} else {
		println!(
			"Hillclimb could not improve with radius {}. {}",
			HILLCLIMB_RADIUS, new_penalty
		);
	}
}

/// Multithreaded randomized hillclimb. Looks at locally optimal choices, and if
/// there are none, shuffles and reruns hillclimb. Repeats for a certain number
/// of iterations per thread.
pub fn rand_hillclimb_threaded(grid: &mut Grid, output_path: &str) {
	let initial_towers = Grid::towers_from_file(output_path);
	for tower in initial_towers {
		grid.add_tower(tower.x, tower.y);
	}
	let old_penalty = grid.penalty();
	let mut grids: Vec<_> = vec![];
	for _ in 0..(num_cpus::get()) {
		grids.push(grid.clone());
	}
	grids
		.par_iter_mut()
		.for_each(|g: &mut Grid| rand_hillclimb(g, output_path, HILLCLIMB_ITERATIONS_PER_THREAD, old_penalty));

	let new_towers = Grid::towers_from_file(output_path);
	grid.remove_all_towers();
	for tower in new_towers {
		grid.add_tower(tower.x, tower.y);
	}
	let new_penalty = grid.penalty();
	if new_penalty < old_penalty {
		println!("{}  {} -> {}", "Improved!".green(), old_penalty, new_penalty);
	} else {
		println!(
			"Randomized hillclimb could not improve in {} iterations with radius {}. {}",
			HILLCLIMB_ITERATIONS_PER_THREAD, HILLCLIMB_RADIUS, new_penalty
		);
	}
}

/// Same as normal hillclimb, except randomizes the grid when reaching a peak,
/// and redoes hillclimb.
fn rand_hillclimb(grid: &mut Grid, output_path: &str, iterations: usize, global_penalty: f64) {
	let mut rng = thread_rng();

	for i in 0..(iterations + 1) {
		loop {
			if !hillclimb_helper(grid, output_path, global_penalty) {
				let pen = grid.penalty();
				if pen < global_penalty {
					println!("Improvement on iteration {}: {} -> {}", i, global_penalty, pen);
					grid.write_solution(output_path);
				} else if i % 10 == 0 {
					// println!("No improvement by iteration {}.", i);
				}
				grid.random_lp_solve(1, rng.gen_range(1..=u32::MAX)); // reinitialize LP-pseudorandom towers
				break;
			}
		}
	}
}

/// Runs hillclimb on this grid and returns whether any improvements were made.
fn hillclimb_helper(grid: &mut Grid, output_path: &str, global_penalty: f64) -> bool {
	fn adjacent_towers(g: &Grid, t: Point, r: u8) -> Vec<Point> {
		// need to change to points_within_naive if want to use different r values.

		let mut adjacent_towers: HashSet<Point> = match r {
			3 | 8 | 10 | 14 => Point::points_within_radius(t, r, g.dimension()).unwrap().clone(),
			_ => Point::points_within_naive(t, r, g.dimension()),
		};
		for (tower, _) in g.get_towers_ref() {
			adjacent_towers.remove(tower);
		}
		adjacent_towers.into_iter().collect()
	}

	let old_penalty = grid.penalty();
	let mut changed = false;
	let old_towers = (*grid.get_towers_ref()).clone();
	let mut rng = thread_rng();
	'outer: for &tower in old_towers.keys() {
		// first sees if valid even without this tower, and if so
		// removes it.
		grid.remove_tower(tower.x, tower.y);
		if grid.is_valid() {
			changed = true;
			grid.write_solution(output_path);
			break 'outer;
		} else {
			grid.add_tower(tower.x, tower.y);
		}

		let mut adj_towers: Vec<Point> = adjacent_towers(grid, tower, HILLCLIMB_RADIUS).into_iter().collect();
		adj_towers.shuffle(&mut rng);
		// now tries to move the tower to a better location
		for adj_tower in adj_towers {
			// change r (third value) if desired
			grid.move_tower(tower, adj_tower);

			if grid.is_valid() {
				let new_penalty = grid.penalty();
				if new_penalty < old_penalty {
					changed = true;
					// println!("{} -> {}, Old: {}, New: {}", tower, adj_tower, old_penalty,
					// new_penalty);
					if new_penalty < global_penalty {
						grid.write_solution(output_path);
					}
					break 'outer;
				}
			}
			grid.move_tower(adj_tower, tower); // undo move
		}
	}
	changed
}

pub fn sort_and_read_penalty(grid: &mut Grid, output_path: &str) {
	let towers = Grid::towers_from_file(output_path);
	for tower in towers {
		grid.add_tower(tower.x, tower.y);
	}
	println!("Penalty: {}", grid.penalty());
	grid.overwrite_with_sorted_solution(output_path);
}

/// Anneal
pub fn simulated_annealing(grid: &mut Grid, output_path: &str) {
	if let Err(ref e) = annealing::run(grid, output_path) {
		println!("{}", e);
		std::process::exit(1);
	}
}

use crate::{grid::Grid, point::Point};
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use stopwatch::Stopwatch;
use rayon::prelude::*;


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
  
  let mut cities = grid
    .get_cities_ref()
    .clone()
    .into_keys()
    .collect::<Vec<Point>>();

  // Continue until cities are covered
  while cities.len() != 0 {
    let mut d: HashMap<Point, u32> = HashMap::new();

    for city in &cities {
      for possible_tower in Point::points_within_radius(*city, grid.service_radius(), grid.dimension()) {
        let counter = d.entry(*possible_tower).or_insert(0);
        *counter += 1
      }
    }

    // Towers to be considered, mapped to added cost
    let mut towers_to_be_considered: HashMap<Point, f64> = HashMap::new();

    // Grab among (us) the towers that cover the most
    let mut ordered_possibles: Vec<(Point, u32)> = d
      .into_iter()
      .collect::<Vec<(Point, u32)>>();
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
    let end = std::cmp::min(
      ((total - index) as f32 * PERCENT_REMAINING) as usize,
      total);

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
      .max_by(|a, b| (*a.1 as u32).cmp(&(*b.1 as u32)) )
      .unwrap().0;

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


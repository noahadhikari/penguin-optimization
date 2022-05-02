use std::io::Write;
use std::sync::{Arc, Mutex};

use argmin::prelude::*;
use argmin::solver::simulatedannealing::{SATempFunc, SimulatedAnnealing};
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::api;
use crate::grid::Grid;
use crate::point::Point;

const INIT_TEMP: f64 = 15.0;

struct Penalty {
	p:   f64,
	rng: Arc<Mutex<Xoshiro256PlusPlus>>,
}

impl Penalty {
	pub fn new(p: f64) -> Self {
		Penalty {
			p,
			rng: Arc::new(Mutex::new(Xoshiro256PlusPlus::from_entropy())),
		}
	}
}

impl ArgminOp for Penalty {
	type Float = f64;
	type Hessian = ();
	type Jacobian = ();
	type Output = f64;
	type Param = Grid;

	fn apply(&self, param: &Grid) -> Result<f64, Error> {
		Ok(param.penalty())
	}

	// Return a valid neighbor of the current state
	fn modify(&self, param: &Grid, temp: f64) -> Result<Grid, Error> {
		Ok(neighbor_one_tower(param))
	}
}

/* Neigboring Functions */

/// Returns a neighbor of the given grid by movine one random tower
/// to a random valid location
fn neighbor_one_tower(param: &Grid) -> Grid {
	let mut grid = param.clone();

	let mut rng = Xoshiro256PlusPlus::from_entropy();

	// Returns random value from a hashmap
	let towers_hashmap = grid.get_towers_ref();
	let mut towers: Vec<Point> = towers_hashmap.keys().map(|p| *p).collect();
	towers.shuffle(&mut rng);

	let towers_to_move = 1;
	let mut valid = false;

	let mut counter = 0;
	while !valid {
		grid = param.clone();
		counter += 1;
		println!("Iteration {}", counter);
		for i in 0..towers_to_move {
			// Get valid points to move the tower
			let tower = towers[i];
			let candidate_points = Point::points_within_naive(tower, 5, grid.dimension());
			let points: Vec<Point> = candidate_points.iter().map(|p| *p).collect();
			let point_to_move_to = points.choose(&mut rng).unwrap();
			if !grid.is_tower_present(*point_to_move_to) && grid.is_on_grid(point_to_move_to.x, point_to_move_to.y) {
				grid.move_tower(tower, *point_to_move_to);
			}
		}
		valid = grid.is_valid();
	}
	grid
}





pub fn run(grid: &mut Grid, output_path: &str) -> Result<(), Error> {
	// Initial grid
	let mut init_grid = grid.clone();
	let sol_towers = Grid::towers_from_file(output_path);
	for point in sol_towers.iter() {
		init_grid.add_tower(point.x, point.y);
	}

	// Cost function
	let operator = Penalty::new(init_grid.penalty());

	let rng = Xoshiro256PlusPlus::from_entropy();

	let solver = SimulatedAnnealing::new(INIT_TEMP, rng)?
		.temp_func(SATempFunc::TemperatureFast)
		// Optional: Reanneal after n iterations (resets temperature to initial temperature)
		.reannealing_fixed(1000)
		// Optional: Reanneal after no accepted solution has been found for n iterations
		.reannealing_accepted(500)
		// Optional: Start reannealing after no new best solution has been found for n iterations
		.reannealing_best(800);

	let res = Executor::new(operator, solver, init_grid)
		.add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
		.max_iters(10000)
		.target_cost(0.0)
		.run()?;

	// Wait a second (lets the logger flush everything before printing again)
	std::thread::sleep(std::time::Duration::from_secs(1));

	// Print result
	println!("{}", res);
	println!("---------------------------------------");
	println!(
		"{} -> {}",
		api::get_penalty_from_file(output_path).unwrap(),
		res.state.best_param.penalty()
	);
	println!("---------------------------------------");
	write_log(
		output_path,
		api::get_penalty_from_file(output_path).unwrap(),
		res.state.best_param.penalty(),
	);
	res.state.best_param.write_solution(output_path);

	Ok(())
}

fn write_log(id: &str, old_pen: f64, new_pen: f64) {
	let mut file = std::fs::OpenOptions::new()
		.append(true)
		.create(true)
		.open("log.txt")
		.unwrap();
	let mut log_string = String::new();
	log_string.push_str(id);
	log_string.push_str(": ");
	log_string.push_str(&old_pen.to_string());
	log_string.push_str(" -> ");
	log_string.push_str(&new_pen.to_string());
	log_string.push_str("\n");
	file.write_all(log_string.as_bytes()).unwrap();
}

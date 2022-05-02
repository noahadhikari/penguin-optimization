use std::cmp::max;
use std::io::Write;
use std::sync::{Arc, Mutex};

use argmin::prelude::*;
use argmin::solver::simulatedannealing::{SATempFunc, SimulatedAnnealing};
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::grid::Grid;
use crate::point::Point;
use crate::{api, solvers};

const INIT_TEMP: f64 = 150.0;
const INIT_CULLING: f64 = 0.1;

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
		// Percent of towers to remove as a func of temperature
		let percent = (temp / INIT_TEMP) * INIT_CULLING;

		let mut grid = param.clone();

		let mut rng = Xoshiro256PlusPlus::from_entropy();

		// Create a random vector of towers
		let towers_hashmap = grid.get_towers_ref();
		let mut towers: Vec<Point> = towers_hashmap.keys().map(|p| *p).collect();
		towers.shuffle(&mut rng);

		let towers_to_move = max((percent * (towers.len() as f64)) as usize, 2);

		// Remove towers from the grid
		for i in 0..towers_to_move {
			let p = towers[i];
			grid.remove_tower(p.x, p.y);
		}

		// Move towers to a random locations such that they cover uncovered cities
		let mut uncovered_cities: Vec<Point> = grid.get_uncovered_cities().iter().map(|p| *p).collect();

		while !grid.is_valid() {
			let city_to_cover = uncovered_cities.pop().unwrap();

			// If city_to_cover is not covered
			if grid.is_city_uncovered(city_to_cover) {
				// Add a tower in a random location that covers city_to_cover
				let candidate_points: Vec<Point> =
					Point::points_within_naive(city_to_cover, grid.service_radius(), grid.dimension())
						.iter()
						.map(|p| *p)
						.collect();
				let point_to_move_to = candidate_points.choose(&mut rng).unwrap();

				grid.add_tower(point_to_move_to.x, point_to_move_to.y);
			}
		}

		Ok(grid)
	}
}

pub fn run(grid: &mut Grid, output_path: &str) -> Result<(), Error> {
	let rng = Xoshiro256PlusPlus::from_entropy();

	// Initial grid
	let mut init_grid = grid.clone();
	solvers::benchmark_greedy(&mut init_grid, output_path);

	// Cost function
	let operator = Penalty::new(init_grid.penalty());

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
		.max_iters(100000)
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

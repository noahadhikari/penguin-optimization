use argmin::prelude::*;
use argmin::solver::simulatedannealing::{SATempFunc, SimulatedAnnealing};
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::sync::{Arc, Mutex};
use crate::grid::Grid;
use crate::point::Point;
use std::default::Default;
use argmin::solver::gradientdescent::SteepestDescent;
use argmin::solver::linesearch::MoreThuenteLineSearch;

const INIT_TEMP: f64 = 15.0;

struct Penalty {
	p: f64,
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
	type Param = Grid;
  type Output = f64;
  type Hessian = ();
  type Jacobian = ();
  type Float = f64;

	fn apply(&self, param: &Grid) -> Result<f64, Error> {
			Ok(param.penalty())
	}

  // Return a valid neighbor of the current state
  fn modify(&self, param: &Grid, temp: f64) -> Result<Grid, Error> {
    
    let percent = temp / (2.0 * INIT_TEMP);

    let mut grid = param.clone();

    let mut rng = Xoshiro256PlusPlus::from_entropy();

    // Returns random value from a hashmap
    let towers_hashmap= grid.get_towers_ref();
    let mut towers: Vec<Point> = towers_hashmap.keys().map(|p| *p).collect();
    towers.shuffle(&mut rng);
    
    let towers_to_move = (percent * (towers.len() as f64)) as usize;
    let mut valid = false;

		let i = 0;
    while !valid {
			println!("Iteration {}", i);
      for i in 0..towers_to_move {
        // Get valid points to move the tower
        let tower = towers[i];
        let candidate_points = Point::points_within_radius(tower, 5, grid.dimension()).unwrap();
        let points: Vec<Point> = candidate_points.iter().map(|p| *p).collect();
        let point_to_move_to = points.choose(&mut rng).unwrap();
        grid.move_tower(tower, *point_to_move_to);
      }
      valid = grid.is_valid();
    }  
    Ok(grid)
  }
}

fn run() -> Result<(), Error> {

	// Initial grid
	let init_param: Grid = Grid::from_file("./inputs/large/123").unwrap();
	
	// Cost function
	let operator = Penalty::new(init_param.penalty());

	let rng = Xoshiro256PlusPlus::from_entropy();

	let solver = SimulatedAnnealing::new(INIT_TEMP, rng)?
		.temp_func(SATempFunc::Boltzmann)
		// Optional: stop if there was no new best solution after n iterations
		.stall_best(1000)
		// Optional: stop if there was no accepted solution after n iterations
		.stall_accepted(1000)
		// Optional: Reanneal after n iterations (resets temperature to initial temperature)
		.reannealing_fixed(1000)
		// Optional: Reanneal after no accepted solution has been found for n iterations
		.reannealing_accepted(500)
		// Optional: Start reannealing after no new best solution has been found for n iterations
		.reannealing_best(800);
	
	let res = Executor::new(operator, solver, init_param);
	Ok(())
}


// fn run() -> Result<(), Error> {
// 	// Define bounds
// 	// let lower_bound: Vec<f64> = vec![-5.0, -5.0];
// 	// let upper_bound: Vec<f64> = vec![5.0, 5.0];

// 	// Define cost function
// 	// let operator = Rosenbrock::new(1.0, 100.0, lower_bound, upper_bound);

// 	// Define initial parameter Grid
// 	let init_param: Grid = Grid::from_file("./inputs/large/123").unwrap();

// 	// Define initial temperature
// 	let temp: f64 = 15.0;

// 	// Seed RNG
// 	let rng = Xoshiro256PlusPlus::from_entropy();

// 	// Set up simulated annealing solver
// 	let solver = SimulatedAnnealing::new(temp, rng)?
// 		.temp_func(SATempFunc::Boltzmann)
// 		// Optional: Stop if there's no best solution after n iterations
// 		.stall_best(1000)
// 		// Optional: Stop if there's no accepted solution after n iterations
// 		.stall_accepted(1000)
// 		// Optional: Reanneal after n iterations (resets temperature to initial temperature)
// 		.reannealing_fixed(1000)
// 		// Optional: Reanneal after no accepted solution has been found for n iterations
// 		.reannealing_accepted(500)
// 		// Optional: Start reannealing after no new best solution has been found for n iterations
// 		.reannealing_best(800);

// 	// Run solver
// 	// let res = Executor::new(init_param, solver, init_param)
// 	// 	.add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
// 	// 	.max_iters(10_000)
// 	// 	.target_cost(0.0)
// 	// 	.run()?;

// 	// Wait a second (lets the logger flush everything before printing again)
// 	std::thread::sleep(std::time::Duration::from_secs(1));

// 	// println!("{}", res);
// 	Ok(}
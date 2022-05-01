use std::collections::HashSet;

use argmin::prelude::*;
use argmin::solver::simulatedannealing::{SATempFunc, SimulatedAnnealing};
use rand::distributions::Uniform;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;
use crate::grid::Grid;
use crate::point::Point;

const INIT_TEMP: f64 = 15.0;


impl ArgminOp for Grid {
  // type Param = &Grid;
  type Param = Grid;
  type Output = f64;
  type Hessian = ();
  type Jacobian = ();
  type Float = f64;


  // Return the cost of the current state 
  fn apply(&self, param: &Grid) -> Result<f64, Error> {
    Ok(param.penalty())
  }

  // Return a valid neighbor of the current state
  fn modify(&self, param: &Grid, temp: f64) -> Result<Grid, Error> {
    
    let percent = temp / (2.0 * INIT_TEMP);

    let mut grid = param.clone();

    let mut rng = Xoshiro256PlusPlus::from_entropy();

    // For now pick a tower at random and move it to a random position

    // Returns random value from a hashmap
    let towers_hashmap= grid.get_towers_ref();
    let mut towers: Vec<Point> = towers_hashmap.keys().map(|p| *p).collect();
    towers.shuffle(&mut rng);
    
    let towers_to_move = (percent * (towers.len() as f64)) as usize;
    let mut valid = false;

    while !valid {
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

// fn neighbor(grid: &Grid) -> Grid {
//     let mut rng = Xoshiro256PlusPlus::from_entropy();
//     let mut new_grid = grid.clone();
//     let mut r = Uniform::new(0, 4);
//     let mut i = r.sample(&mut rng);
//     while i == grid.last_move {
//         i = r.sample(&mut rng);
//     }
//     new_grid.move_to(i);
//     new_grid
// }



fn anneal(grid: &Grid) -> Grid {
    let mut rng = Xoshiro256PlusPlus::from_entropy();


    let res = Executor::new(cost, solver)
}

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::lp::GridProblem;
use crate::point::Point;

// A Grid which we place towers and cities on.
#[derive(Clone)]
pub struct Grid {
	dimension:      u8,
	service_radius: u8,
	penalty_radius: u8,

	// Mapping from <coordinates of towers, coordinates of other towers within penalty radius>.
	// i.e. < (2, 3), {(5, 6), (7, 8)} >
	towers: HashMap<Point, HashSet<Point>>,

	// Mapping from <coordinates of cities, towers that cover it>.
	// i.e. < (4, 4), {(1, 2), (3, 4)} >
	cities: HashMap<Point, HashSet<Point>>,
}

impl fmt::Debug for Grid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if f.alternate() {
			// pretty print
			write!(
				f,
				"Grid {{ \n\nPenalty: {}\nValid: {}\n\ndimension: {}, service_radius: {}, penalty_radius: {},\n\ntowers: \
				 {:#?},\n\ncities: {:#?} \n\n}}",
				self.penalty(),
				self.is_valid(),
				self.dimension,
				self.service_radius,
				self.penalty_radius,
				self.towers,
				self.cities
			)
		} else {
			// standard print

			write!(
				f,
				"Grid {{ Penalty: {}, Valid: {}, dimension: {}, service_radius: {}, penalty_radius: {}, towers: {:?}, cities: \
				 {:?} }}",
				self.penalty(),
				self.is_valid(),
				self.dimension,
				self.service_radius,
				self.penalty_radius,
				self.towers,
				self.cities
			)
		}
	}
}

/// Pretty printer for Grid.
impl fmt::Display for Grid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Penalty: {}\n", self.penalty());
		for y in (0..self.dimension).rev() {
			for x in 0..self.dimension {
				let p = Point::new(x as i32, y as i32);
				if self.towers.contains_key(&p) && self.cities.contains_key(&p) {
					write!(f, "¢"); // city and tower at same point
				} else if self.towers.contains_key(&p) {
					write!(f, "t")?; // tower at this point
				} else if self.cities.contains_key(&p) {
					write!(f, "c")?; // city at this point
				} else {
					write!(f, "·")?; // nothing at this point
				}
				write!(f, " ")?;
			}
			write!(f, "\n")?;
		}
		Ok(())
	}
}

impl Grid {
	/// Creates and returns a new Grid of the given dimension, service_radius, and
	/// penalty radius.
	pub fn new(dimension: u8, service_radius: u8, penalty_radius: u8) -> Self {
		Grid {
			dimension,
			service_radius,
			penalty_radius,
			towers: HashMap::new(),
			cities: HashMap::new(),
		}
	}

	pub fn new_dummy_grid() -> Grid {
		Grid::new(0, 0, 0)
	}

	/// Returns the total penalty P of this Grid.
	pub fn penalty(&self) -> f64 {
		let mut penalty = 0.0;
		for penalized in self.towers.values() {
			let w_j = penalized.len() as f64;
			penalty += (0.17 * w_j).exp();
		}
		170.0 * penalty
	}

	/// Returns whether the towers in this Grid cover all cities.
	pub fn is_valid(&self) -> bool {
		self.cities.values().all(|c| c.len() > 0)
	}

	/// Adds a city at (x, y) to this Grid, if it does not already exist.
	/// Can only add cities if no towers have been placed yet.
	pub fn add_city(&mut self, x: i32, y: i32) {
		assert!(self.towers.len() == 0, "Cannot add cities after placing towers.");
		self.check_coordinates(x, y);
		let c = Point::new(x, y);
		assert!(
			!self.cities.contains_key(&c),
			"Cannot add city at {:?} because it already exists.",
			c
		);
		self.cities.insert(c, HashSet::new());
	}

	/// Adds a tower at (x, y) to this Grid, if it does not already exist.
	pub fn add_tower(&mut self, x: i32, y: i32) {
		self.check_coordinates(x, y);
		let t: Point = Point::new(x, y);
		assert!(
			!self.towers.contains_key(&t),
			"Cannot add tower at {:?} because it already exists.",
			t
		);
		self.update_towers_add(t); // implicitly adds the tower to the grid
		self.update_cities_add(t);
	}

	/// Used upon adding a tower T.
	/// Updates the penalized towers for each tower within the penalty radius of
	/// T.
	fn update_towers_add(&mut self, p: Point) {
		let penalized = Point::points_within_radius(p, self.penalty_radius, self.dimension);

		let mut adj_towers = HashSet::new();
		for (&tower, set) in self.towers.iter_mut() {
			if penalized.contains(&tower) && tower != p {
				set.insert(p);
				adj_towers.insert(tower);
			}
		}
		self.towers.insert(p, adj_towers);
	}

	/// Used upon adding a tower T.
	/// Adds T to the covering towers for each city within the service radius of
	/// T.
	fn update_cities_add(&mut self, t: Point) {
		let coverage = Point::points_within_radius(t, self.service_radius, self.dimension);
		// println!("t = {}, \n coverage = {:#?}", t, coverage);

		for (c, ts) in self.cities.iter_mut() {
			if coverage.contains(&c) && !ts.contains(&t) {
				ts.insert(t);
			}
		}
	}

	/// Removes the tower at (x, y) from this Grid, if it exists.
	/// Also updates the respective tower and city coverage.
	pub fn remove_tower(&mut self, x: i32, y: i32) {
		self.check_coordinates(x, y);
		let p: Point = Point::new(x, y);
		assert!(
			self.towers.contains_key(&p),
			"Cannot remove tower at {:?} because it does not exist.",
			p
		);
		self.update_towers_remove(p); // implicitly removes the tower from the grid
		self.update_cities_remove(p);
	}

	/// Used upon removing a tower T.
	/// Updates the penalized towers for each tower within the penalty radius of
	/// T.
	fn update_towers_remove(&mut self, t: Point) {
		for (_t, others) in self.towers.iter_mut() {
			others.remove(&t);
		}
		self.towers.remove(&t);
	}

	/// Used upon removing a tower T.
	/// Removes T from the covering towers for each city within the service radius
	/// of T.
	fn update_cities_remove(&mut self, t: Point) {
		for (_c, ts) in self.cities.iter_mut() {
			ts.remove(&t); // does nothing if called on city uncovered by T
		}
	}

	/// Asserts that the given coordinates are within this Grid.
	fn check_coordinates(&self, x: i32, y: i32) {
		assert!(
			x >= 0 && y >= 0 && x < self.dimension as i32 && y < self.dimension as i32,
			"Coordinates off the edge of grid: ({}, {}) for grid dimension {}",
			x,
			y,
			self.dimension
		);
	}

	/// Returns the file output string of this entire Grid.
	pub fn output(&self) -> String {
		let mut res = format!("# Penalty = {}\n", self.penalty());
		res += &(self.towers.len().to_string() + "\n");
		for (point, _) in self.towers.iter() {
			res += &(point.file_string() + "\n");
		}
		res
	}

	pub fn get_cities_ref(&self) -> &HashMap<Point, HashSet<Point>> {
		&self.cities
	}

	pub fn service_radius(&self) -> u8 {
		self.service_radius
	}

	pub fn penalty_radius(&self) -> u8 {
		self.penalty_radius
	}

	pub fn dimension(&self) -> u8 {
		self.dimension
	}
	pub fn get_towers_ref(&self) -> &HashMap<Point, HashSet<Point>> {
		&self.towers
	}

	pub fn replace_all_towers(&mut self, towers: HashMap<Point, HashSet<Point>>) {
		if self.towers == towers {
			return;
		}
		self.remove_all_towers();
		for (point, _) in towers.iter() {
			self.add_tower(point.x, point.y);
		}
	}

	pub fn set_service_radius(&mut self, serv_radius: u8) {
		self.service_radius = serv_radius;
	}

	pub fn set_penalty_radius(&mut self, pen_radius: u8) {
		self.penalty_radius = pen_radius;
	}

	pub fn set_dimension(&mut self, dim: u8) {
		self.dimension = dim;
	}

	pub fn remove_all_towers(&mut self) {
		self.towers.clear();
		for (_, covered) in self.cities.iter_mut() {
			covered.clear();
		}
	}

	/// Randomly solves the Grid using LP up until the max time and
	/// returns (penalty, towers).
	pub fn random_lp_solve(&mut self, max_time: u32, seed: u32) -> f64 {
		let mut city_keys = HashSet::new();
		for (&c, _) in self.cities.iter() {
			city_keys.insert(c);
		}

		// use rand::{thread_rng, Rng};
		// let mut rng = thread_rng();
		self.remove_all_towers();
		let problem = GridProblem::new_randomized(
			self.dimension,
			self.service_radius,
			self.penalty_radius,
			city_keys,
			max_time,
			seed,
		);
		let tower_soln = problem.tower_solution();
		for t in tower_soln {
			self.add_tower(t.x, t.y);
		}
		self.penalty()
	}

	/// Destructively (changes the grid's tower configuration) solves the Grid
	/// using the LP.
	pub fn lp_solve(&mut self, max_time: u32) {
		assert!(
			self.towers.len() == 0,
			"Cannot solve a grid with towers already placed."
		);

		let mut city_keys = HashSet::new();
		for (&c, _) in self.cities.iter() {
			city_keys.insert(c);
		}

		let problem = GridProblem::new(
			self.dimension,
			self.service_radius,
			self.penalty_radius,
			city_keys,
			max_time,
		);

		for t in problem.tower_solution() {
			self.add_tower(t.x, t.y);
		}
	}
}

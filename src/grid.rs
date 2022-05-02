use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::{fmt, io};

use serde::{Serialize, Deserialize};

use crate::api;
use crate::lp::GridProblem;
use crate::point::Point;

// A Grid which we place towers and cities on.
#[derive(Clone, Serialize, Deserialize)]
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
		write!(f, "Penalty: {}\n", self.penalty())?;
		for y in (0..self.dimension).rev() {
			for x in 0..self.dimension {
				let p = Point::new(x as i32, y as i32);
				if self.towers.contains_key(&p) && self.cities.contains_key(&p) {
					// write!(f, "¢"); // city and tower at same point
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

	/// Deeply clone the grid
	pub fn clone(&self) -> Self {
		let mut new_grid = Grid::new(self.dimension, self.service_radius, self.penalty_radius);
		new_grid.towers = self.towers.clone();
		new_grid.cities = self.cities.clone();
		new_grid
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
		let penalized = Point::points_within_radius(p, self.penalty_radius, self.dimension).unwrap();

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
		let coverage = Point::points_within_radius(t, self.service_radius, self.dimension).unwrap();
		// println!("t = {}, \n coverage = {:#?}", t, coverage);

		for (c, ts) in self.cities.iter_mut() {
			if (c == &t) || (coverage.contains(c) && !ts.contains(&t)) {
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

	/// Returns if a tower is present on the given point
	pub fn is_tower_present(&self, p: Point) -> bool {
		self.towers.contains_key(&p)
	}	

	/// Moves a tower from P = (x, y) to Q = (x', y').
	/// Fails if tower at P does not exist or if tower at Q already exists.
	pub fn move_tower(&mut self, p: Point, q: Point) {
		assert!(
			self.towers.contains_key(&p),
			"Cannot move tower from {:?} because it does not exist.",
			p
		);
		assert!(
			!self.towers.contains_key(&q),
			"Cannot move tower to {:?} because there is already a tower there.",
			q
		);
		self.remove_tower(p.x, p.y);
		self.add_tower(q.x, q.y);
	}

	/// Asserts that the given coordinates are within this Grid.
	fn check_coordinates(&self, x: i32, y: i32) {
		assert!(
			self.is_on_grid(x, y),
			"Coordinates off the edge of grid: ({}, {}) for grid dimension {}",
			x,
			y,
			self.dimension
		);
	}

	/// Returns whether (x, y) is within the grid.
	pub fn is_on_grid(&self, x: i32, y: i32) -> bool {
		x >= 0 && y >= 0 && x < self.dimension as i32 && y < self.dimension as i32
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

	/// Returns the grid created from the passed in input file.
	pub fn from_file(path: &str) -> io::Result<Grid> {
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

	// Write self to a file as a solution
	pub fn write_solution(&self, output_path: &str) {
		assert!(self.is_valid(), "Not a valid solution");
		// Only overwrite if solution is better than what we currently have
		let mut existing_penalty = 0.;
		if Path::new(output_path).is_file() {
			while existing_penalty == 0. {
				existing_penalty = api::round(api::get_penalty_from_file(output_path).unwrap_or(0.));
			}

			if self.penalty() >= existing_penalty {
				return;
			}
		}
		
		let data = self.output();
		let mut f = OpenOptions::new()
			.write(true)
			.truncate(true)
			.create(true)
			.open(output_path)
			.expect("Unable to open file");
		f.write_all(data.as_bytes()).expect("Unable to write data");
	}

	/// Randomly solves the Grid using LP up until the max time and
	/// returns penalty.
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

	pub fn towers_from_file(path: &str) -> HashSet<Point> {
		let mut towers = HashSet::new();
		let file = File::open(path).expect("Unable to open file");
		let reader = BufReader::new(file);

		let mut i: i32 = 0;
		for line in reader.lines() {
			match i {
				0 => {
					i += 1;
					continue;
				}
				1 => {
					i += 1;
					continue;
				}
				_ => {
					if let Ok(l) = line {
						let vec: Vec<&str> = l.split_whitespace().collect();
						let x = vec.get(0).unwrap().parse::<i32>().unwrap();
						let y = vec.get(1).unwrap().parse::<i32>().unwrap();
						towers.insert(Point::new(x, y));
					}
				}
			}
		}
		towers
	}
}

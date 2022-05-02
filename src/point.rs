use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

// Static preprocessed data for points within radii.
lazy_static! {
	static ref PEN_S: HashMap<Point, HashSet<Point>> = preprocess::load("small", "penalty");
	static ref PEN_M: HashMap<Point, HashSet<Point>> = preprocess::load("medium", "penalty");
	static ref PEN_L: HashMap<Point, HashSet<Point>> = preprocess::load("large", "penalty");
	static ref SVC_S: HashMap<Point, HashSet<Point>> = preprocess::load("small", "service");
	static ref SVC_M: HashMap<Point, HashSet<Point>> = preprocess::load("medium", "service");
	static ref SVC_L: HashMap<Point, HashSet<Point>> = preprocess::load("large", "service");
}

// Preprocessing module for points within radii.
pub mod preprocess {
	use std::fs::{File, OpenOptions};
	use std::io::prelude::*;
	use std::io::BufReader;
	use std::path::Path;

	use super::*;


	/// Writes out the preprocessing data for all combinations of size and cover.
	pub fn setup_persistence() {
		let options = vec![
			("small", "penalty"),
			("medium", "penalty"),
			("large", "penalty"),
			("small", "service"),
			("medium", "service"),
			("large", "service"),
		];
		for (size, cover) in options {
			create(size, cover);
		}
	}
	/// Writes out the coverage points for the given size and cover, i.e. penalty
	/// or service.
	fn create(size: &str, cover: &str) {
		let output_path = match (size, cover) {
			("small", "penalty") => "./preprocess/penalty/small.txt",
			("medium", "penalty") => "./preprocess/penalty/medium.txt",
			("large", "penalty") => "./preprocess/penalty/large.txt",
			("small", "service") => "./preprocess/service/small.txt",
			("medium", "service") => "./preprocess/service/medium.txt",
			("large", "service") => "./preprocess/service/large.txt",
			_ => panic!("Invalid size or cover"),
		};

		let r: u8 = match (size, cover) {
			("small", "penalty") => 8,
			("medium", "penalty") => 10,
			("large", "penalty") => 14,
			("small", "service") => 3,
			("medium", "service") => 3,
			("large", "service") => 3,
			_ => panic!("Invalid size or cover"),
		};

		let dim: u8 = match size {
			"small" => 30,
			"medium" => 50,
			"large" => 100,
			_ => panic!("Invalid size"),
		};

		assert!(
			!Path::new(output_path).exists(),
			"Point preprocessing for {} already exists.",
			size
		);

		let mut map: HashMap<Point, HashSet<Point>> = HashMap::new();
		for i in 0..dim {
			for j in 0..dim {
				let p = Point::new(i.into(), j.into());
				let mut points_within = Point::points_within_naive(p, r, dim);
				points_within.remove(&p);
				map.insert(p, points_within);
			}
		}
		let s = format! {"{:#?}", map};
		let mut file = OpenOptions::new().write(true).create(true).open(output_path).unwrap();
		file.write_all(s.as_bytes()).unwrap();
	}

	/// Loads the preprocessed points for the given size (small, medium, large)
	/// and cover, i.e. penalty or service
	pub fn load(size: &str, cover: &str) -> HashMap<Point, HashSet<Point>> {
		let input_path = match (size, cover) {
			("small", "penalty") => "./preprocess/penalty/small.txt",
			("medium", "penalty") => "./preprocess/penalty/medium.txt",
			("large", "penalty") => "./preprocess/penalty/large.txt",
			("small", "service") => "./preprocess/service/small.txt",
			("medium", "service") => "./preprocess/service/medium.txt",
			("large", "service") => "./preprocess/service/large.txt",
			_ => panic!("Invalid size or cover"),
		};

		assert!(
			Path::new(input_path).exists(),
			"Input path does not exist: {}",
			input_path
		);
		let file = File::open(input_path).unwrap();
		let reader = BufReader::new(file);
		let mut result = HashMap::new();
		let mut point = Point::new(-69, -69);
		let mut within: HashSet<Point> = HashSet::new();
		let mut found = false;
		use regex::Regex;
		// Regex pattern matching points (x, y)
		let re = Regex::new(r"\((\d+), (\d+)\)").unwrap();

		for line in reader.lines() {
			let line = line.unwrap();
			let line = line.trim();
			if line.eq("}") {
				result.insert(point, within.clone());
				return result;
			}
			if line.len() <= 2 {
				continue;
			}
			for cap in re.captures_iter(line) {
				let x = cap[1].parse::<i32>().unwrap();
				let y = cap[2].parse::<i32>().unwrap();
				let p = Point::new(x, y);
				if line.chars().last().unwrap() == '{' {
					if found {
						result.insert(point, within.clone());
						within.clear();
					}
					point = p;
				} else {
					within.insert(p);
				}
				found = true;
			}
		}

		result
	}
}


/// Represents a lattice point on the grid. Has integer x-y coordinates.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

impl fmt::Debug for Point {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl fmt::Display for Point {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl Point {
	/// Creates and returns a new Point with the given x and y coordinates.
	pub fn new(x: i32, y: i32) -> Self {
		Point { x, y }
	}

	/// Returns the Euclidean distance between two points.
	fn dist(p1: &Point, p2: &Point) -> f64 {
		(((p1.x - p2.x).pow(2) + (p1.y - p2.y).pow(2)) as f64).sqrt()
	}

	/// Returns the Euclidean distance between this point and the given point.
	fn dist_to(&self, p: &Point) -> f64 {
		Point::dist(self, p)
	}

	/// Returns the file string form of this point, e.g. (3, 4) -> "3 4".
	pub fn file_string(&self) -> String {
		self.x.to_string() + " " + &self.y.to_string()
	}

	pub fn get_x(&self) -> i32 {
		self.x
	}

	pub fn get_y(&self) -> i32 {
		self.y
	}

	pub fn points_within_naive(p: Point, r: u8, dim: u8) -> HashSet<Point> {
		let mut result = HashSet::new();
		let r = r as i32;
		for i in -r..(r + 1) {
			for j in -r..(r + 1) {
				if Self::within(r, p.x, p.y, p.x + i, p.y + j, dim) {
					result.insert(Self::new(p.x + i, p.y + j));
				}
			}
		}

		result
	}

	/// Returns a set of all the grid points within the given radius of the given
	/// point.
	pub fn points_within_radius(p: Point, r: u8, dim: u8) -> Result<&'static HashSet<Point>, &'static str> {
		let result = match (dim, r) {
			(30, 8) => PEN_S.get(&p),
			(50, 10) => PEN_M.get(&p),
			(100, 14) => PEN_L.get(&p),
			(30, 3) => SVC_S.get(&p),
			(50, 3) => SVC_M.get(&p),
			(100, 3) => SVC_L.get(&p),
			_ => None,
		};
		// println!("{}: {:?}", p, result);
		match result {
			Some(result) => Ok(result),
			None => panic!("Didn't find preprocessed"),
		}
	}

	/// Returns whether (x2, y2) is within r units of (x1, y1) and within this
	/// Grid.
	fn within(r: i32, x1: i32, y1: i32, x2: i32, y2: i32, d: u8) -> bool {
		if x2 < 0 || x2 >= d as i32 || y2 < 0 || y2 >= d as i32 {
			return false;
		}
		(x1 - x2).pow(2) + (y1 - y2).pow(2) <= r.pow(2)
	}
}

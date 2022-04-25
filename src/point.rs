use std::fmt;
use std::collections::HashSet;

/// Represents a lattice point on the grid. Has integer x-y coordinates.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
	pub fn new(x: i32, y: i32) -> Point {
		Point { x, y }
	}

	/// Returns the Euclidean distance between two points.
	pub fn dist(p1: &Point, p2: &Point) -> f64 {
		(((p1.x - p2.x).pow(2) + (p1.y - p2.y).pow(2)) as f64).sqrt()
	}

	/// Returns the Euclidean distance between this point and the given point.
	pub fn dist_to(&self, p: &Point) -> f64 {
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

	/// Returns a set of all the grid points within the given radius of the given
	/// point.
	pub fn points_within_radius(p: Point, r: u8, dim: u8) -> HashSet<Point> {
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

	/// Returns whether (x2, y2) is within r units of (x1, y1) and within this
	/// Grid.
	fn within(r: i32, x1: i32, y1: i32, x2: i32, y2: i32, d: u8) -> bool {
		if x2 < 0 || x2 >= d as i32 || y2 < 0 || y2 >= d as i32 {
			return false;
		}
		(x1 - x2).pow(2) + (y1 - y2).pow(2) <= r.pow(2)
	}
}

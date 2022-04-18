use std::collections::{HashSet, HashMap};
use std::fmt;

// A Grid which we place towers and cities on.
// #[derive(Debug)]
pub struct Grid {
    dimension: u8,
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
        if f.alternate() { // pretty print
            write!(f, "Grid {{ \n\nPenalty: {}\nValid: {}\n\ndimension: {}, service_radius: {}, penalty_radius: {},\n\ntowers: {:#?},\n\ncities: {:#?} \n\n}}",
            self.penalty(), 
            self.is_valid(), 
            self.dimension, 
            self.service_radius, 
            self.penalty_radius, 
            self.towers,
            self.cities)

        } else { // standard print
            
            write!(f, "Grid {{ Penalty: {}, Valid: {}, dimension: {}, service_radius: {}, penalty_radius: {}, towers: {:?}, cities: {:?} }}",
            self.penalty(),
            self.is_valid(), 
            self.dimension, 
            self.service_radius, 
            self.penalty_radius, 
            self.towers, 
            self.cities)
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Penalty: {}\n", self.penalty());
        for y in (0..self.dimension).rev() {
            for x in 0..self.dimension {
                let point = Point::new(x as i32, y as i32);
                if self.towers.contains_key(&point) && self.cities.contains_key(&point) {
                    write!(f, "¢"); //ţ∉ç¢
                } else if self.towers.contains_key(&point) {
                    write!(f, "t")?;
                } else if self.cities.contains_key(&point) {
                    write!(f, "c")?;
                } else {
                    write!(f, "·")?;
                }
                write!(f, " ")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Grid {
    /// Creates and returns a new Grid of the given dimension, service_radius, and penalty radius.
    pub fn new(dimension: u8, service_radius: u8, penalty_radius: u8) -> Grid {
        Grid {
            dimension,
            service_radius,
            penalty_radius,
            towers: HashMap::new(),
            cities: HashMap::new(),
        }
    }

    /// Returns the total penalty P of this Grid.
    pub fn penalty(&self) -> f64 {
        let mut penalty = 0.0;
        for penalized in self.towers.values() {
            let w_j = penalized.len();
            penalty += (0.17 * w_j as f64).exp();
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
        let p = Point::new(x, y);
        assert!(!self.cities.contains_key(&p), "Cannot add city at {:?} because it already exists.", p);
        self.cities.insert(p, HashSet::new());
    }

    /// Adds a tower at (x, y) to this Grid, if it does not already exist.
    pub fn add_tower(&mut self, x: i32, y: i32) {
        self.check_coordinates(x, y);
        let p: Point = Point::new(x, y);
        self.update_towers(p); //implicitly adds the tower to the grid
        self.update_cities(p);
    }

    /// Adds all towers passed in to this Grid, if they do not already exist.
    // macro_rules! add_towers {
    //     ($($x:expr),*) => {
    //         {
    //             $(self.add_tower($x);)*
    //         }
    //     }
    // }
    
    /// Used upon adding a tower P.
    /// Updates the w_j value for each tower within the penalty radius of P.
    fn update_towers(&mut self, p: Point) {
        let penalized = self.points_within_radius(p, self.penalty_radius);

        let mut adj_towers = HashSet::new();
        for (&tower, set) in self.towers.iter_mut() {
            if penalized.contains(&tower) && tower != p {
                set.insert(p);
                adj_towers.insert(tower);
            }
        }
        self.towers.insert(p, adj_towers);
    }

    /// Used upon adding a tower P.
    /// Adds P to the covering towers for each city within the service radius of P.
    fn update_cities(&mut self, p: Point) {
        let coverage = self.points_within_radius(p, self.service_radius);
        // println!("p = {}, \n coverage = {:#?}", p, coverage);

        for (c, ts) in self.cities.iter_mut() {
            if coverage.contains(&c) && !ts.contains(&p) {
                ts.insert(p);
            }
        } 
    }
        
    /// Removes the tower at (x, y) from this Grid, if it exists.
    pub fn remove_tower(&mut self, x: i32, y: i32) {
        self.check_coordinates(x, y);
        self.towers.remove(&Point::new(x, y));
    }

    /// Asserts that the given coordinates are within this Grid.
    fn check_coordinates(&self, x: i32, y: i32) {
        assert!(x >= 0 && y >= 0 && x < self.dimension as i32 && y < self.dimension as i32, 
            "Coordinates off the edge of grid: ({}, {}) for grid dimension {}", x, y, self.dimension);
    }

    /// Returns a vector of all the grid points within the given radius of the given point.
    fn points_within_radius(&self, p: Point, r: u8) -> HashSet<Point> {
        let mut result = HashSet::new();
        let r = r as i32;
        for i in -r..r {
            for j in -r..r {
                if self.within(r, p.x, p.y, p.x + i, p.y + j) {
                    result.insert(Point::new(p.x + i, p.y + j));
                }
            }
        }

        result
    }

		pub fn output(&self) -> String {
			let mut res = format!("# Penalty = {}\n", self.penalty());
			res += &(self.towers.len().to_string() + "\n");
			for (point, _) in self.towers.iter() {
				res += &(point.file_string() + "\n");
			}
			res
		}

    /// Returns whether (x2, y2) is within r units of (x1, y1) and within this Grid.
    fn within(&self, r: i32, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        if x2 < 0 || x2 > self.dimension as i32 || y2 < 0 || y2 > self.dimension as i32 {
            return false;
        }
        (x1 - x2).pow(2) + (y1 - y2).pow(2) <= r.pow(2)
    }

		pub fn get_cities(&self) -> &HashMap<Point, HashSet<Point>> {
			&self.cities
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
}

/// Represents a lattice point on the grid. Has integer x-y coordinates.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    x: i32,
    y: i32,
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
    fn new(x: i32, y: i32) -> Point {
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

		fn file_string(&self) -> String {
			self.x.to_string() + " " + &self.y.to_string()	
		}

		pub fn get_x(&self) -> i32 {
			self.x
		}

		pub fn get_y(&self) -> i32 {
			self.y
		}

}

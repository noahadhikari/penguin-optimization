use std::collections::HashSet;

// A Grid which we place towers and cities on.
#[derive(Debug)]
pub struct Grid {
    dimension: usize,
    service_radius: u8,
    penalty_radius: u8,

    // The coordinates of towers.
    towers: HashSet<Point>,

    // The coordinates of cities.
    cities: HashSet<Point>,
}

impl Grid {
    /// Creates and returns a new Grid of the given dimension, service_radius, and penalty radius.
    pub fn new(dimension: usize, service_radius: u8, penalty_radius: u8) -> Grid {
        Grid {
            dimension,
            service_radius,
            penalty_radius,
            towers: HashSet::new(),
            cities: HashSet::new(),
        }
    }

    /// Returns the total penalty P of this Grid.
    pub fn total_penalty(&self) -> f64 {
        let mut total_penalty = 0.0;
        for tower1 in self.towers.iter() {
            let mut w_j = 0;
            for tower2 in self.towers.iter() {
                if tower1 != tower2 {
                    if tower1.dist_to(tower2) <= self.penalty_radius as f64 {
                        w_j += 1;
                    }
                }
            }
            total_penalty += (0.17 * w_j as f64).exp();
        }
        170.0 * total_penalty
    }

    /// Returns whether the towers in this Grid cover all cities.
    pub fn is_valid(&self) -> bool {
        for city in self.cities.iter() {
            let mut covered = false;
            for tower in self.towers.iter() {
                if city.dist_to(tower) <= self.service_radius as f64 {
                    covered = true;
                    break;
                }
            }
            if !covered {
                return false;
            }
        }
        true
    }

    /// Adds a city at (x, y) to this Grid, if it does not already exist.
    pub fn add_city(&mut self, x: usize, y: usize) {
        Self::check_coordinates(x, y, self.dimension);
        self.cities.insert(Point::new(x, y));
    }

    /// Adds a tower at (x, y) to this Grid, if it does not already exist.
    pub fn add_tower(&mut self, x: usize, y: usize) {
        Self::check_coordinates(x, y, self.dimension);
        self.towers.insert(Point::new(x, y));
    }

    /// Removes the tower at (x, y) from this Grid, if it exists.
    pub fn remove_tower(&mut self, x: usize, y: usize) {
        Self::check_coordinates(x, y, self.dimension);
        self.towers.remove(&Point::new(x, y));
    }

    pub fn set_service_radius(&mut self, serv_radius: u8) {
        self.service_radius = serv_radius;
    }

    pub fn set_penalty_radius(&mut self, pen_radius: u8) {
        self.penalty_radius = pen_radius;
    }

    pub fn set_dimension(&mut self, dim: usize) {
        self.dimension = dim;
    }

    fn check_coordinates(x: usize, y: usize, dimension: usize) {
        assert!(x < dimension && y < dimension,
            "Coordinates off the edge of grid: ({}, {}) for grid dimension {}", x, y, dimension);
    }
}

/// Represents a lattice point on the grid. Has integer x-y coordinates.
#[derive(Debug, PartialEq, Eq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    /// Creates and returns a new Point with the given x and y coordinates.
    fn new(x: usize, y: usize) -> Point {
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
}

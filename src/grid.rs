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

/// Pretty printer for Grid.
impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Penalty: {}\n", self.penalty());
        for y in (0..self.dimension).rev() {
            for x in 0..self.dimension {
                let p = Point::new(x as i32, y as i32);
                if self.towers.contains_key(&p) && self.cities.contains_key(&p) {
                    write!(f, "¢"); //city and tower at same point
                } else if self.towers.contains_key(&p) {
                    write!(f, "t")?; //tower at this point
                } else if self.cities.contains_key(&p) {
                    write!(f, "c")?; // city at this point
                } else {
                    write!(f, "·")?; //nothing at this point
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
        let c = Point::new(x, y);
        assert!(!self.cities.contains_key(&c), "Cannot add city at {:?} because it already exists.", c);
        self.cities.insert(c, HashSet::new());
    }

    /// Adds a tower at (x, y) to this Grid, if it does not already exist.
    pub fn add_tower(&mut self, x: i32, y: i32) {
        self.check_coordinates(x, y);
        let t: Point = Point::new(x, y);
        assert!(!self.towers.contains_key(&t), "Cannot add tower at {:?} because it already exists.", t);
        self.update_towers_add(t); //implicitly adds the tower to the grid
        self.update_cities_add(t);
    }

    /// Used upon adding a tower T.
    /// Updates the penalized towers for each tower within the penalty radius of T.
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
    /// Adds T to the covering towers for each city within the service radius of T.
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
        assert!(self.towers.contains_key(&p), "Cannot remove tower at {:?} because it does not exist.", p);
        self.update_towers_remove(p); //implicitly removes the tower from the grid
        self.update_cities_remove(p);
    }

    /// Used upon removing a tower T.
    /// Updates the penalized towers for each tower within the penalty radius of T.
    fn update_towers_remove(&mut self, t: Point) {
        for (_t, others) in self.towers.iter_mut() {
            others.remove(&t);
        }
        self.towers.remove(&t);
    }

    /// Used upon removing a tower T.
    /// Removes T from the covering towers for each city within the service radius of T.
    fn update_cities_remove(&mut self, t: Point) {
        for (_c, ts) in self.cities.iter_mut() {
            ts.remove(&t); //does nothing if called on city uncovered by T
        } 
    }


    /// Asserts that the given coordinates are within this Grid.
    fn check_coordinates(&self, x: i32, y: i32) {
        assert!(x >= 0 && y >= 0 && x < self.dimension as i32 && y < self.dimension as i32, 
            "Coordinates off the edge of grid: ({}, {}) for grid dimension {}", x, y, self.dimension);
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



    /// Destructively (changes the grid's tower configuration) solves the Grid using the LP.
    pub fn solve(&mut self) {
        assert!(self.towers.len() == 0, "Cannot solve a grid with towers already placed.");
        use solver::GridProblem;
        
        let mut city_keys = HashSet::new();
        for (&c, _) in self.cities.iter() {
            city_keys.insert(c);
        }

        let problem = GridProblem::new(
            self.dimension, 
            self.penalty_radius, 
            self.service_radius, 
            city_keys
        );

        for t in problem.tower_solution() {
            self.add_tower(t.x, t.y);
        };


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

    /// Returns the file string form of this point, e.g. (3, 4) -> "3 4".
    fn file_string(&self) -> String {
        self.x.to_string() + " " + &self.y.to_string()	
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    /// Returns a set of all the grid points within the given radius of the given point.
    pub fn points_within_radius(p: Point, r: u8, dim: u8) -> HashSet<Point> {
        let mut result = HashSet::new();
        let r = r as i32;
        for i in -r..r {
            for j in -r..r {
                if Self::within(r, p.x, p.y, p.x + i, p.y + j, dim) {
                    result.insert(Self::new(p.x + i, p.y + j));
                }
            }
        }

        result
    }

    /// Returns whether (x2, y2) is within r units of (x1, y1) and within this Grid.
    fn within(r: i32, x1: i32, y1: i32, x2: i32, y2: i32, d: u8) -> bool {
        if x2 < 0 || x2 > d as i32 || y2 < 0 || y2 > d as i32 {
            return false;
        }
        (x1 - x2).pow(2) + (y1 - y2).pow(2) <= r.pow(2)
    }

}





mod solver {

    /// Idea: Because penalty is monotonic ish, can try to minimize a linear penalty to use LP.
    /// 
    /// let z_{ij} = {0, 1} correspond to whether or not a tower is placed at (i, j)
    /// 
    /// for each city, ensure that at least one tower is covering it.
    /// i.e. for each c_ij, sum of z_{ij} in coverage(c_ij) >= 1
    /// 
    /// for each point in z, want to calculate penalty for that point, but only if the tower is actually there.
    /// let p_{ij,kl} = penalty at position ij for position kl only if tower z_ij is present and z_kl exists for all
    ///     kl in the penalty coverage of ij.
    /// p_ijkl = z_kl if z_ij else 0
    /// -> z_ij AND z_kl
    /// -> p_ijkl <= z_kl, p_ijkl <= z_ij, p_ijkl >= z_ij + z_kl - 1.
    /// 
    /// TODO: investigate - can we have piecewise linear (leaky relu) for each p_i? Then sum over p_i to get total penalty. 
    ///     will ignore for now.
    /// 
    /// all variables are binary except the total penalty (maybe unnecessary, but P = sum_ij p_ij).
    /// minimize sum of p_ij (== P).
    /// 
    /// ------------------------------
    /// 
    /// total number of variables is on the order of R^2 * d^2.
    
    use super::*;
    use good_lp::variable::ProblemVariables;
    use good_lp::constraint::Constraint;
    use good_lp::{default_solver, constraint, variable, variables, Expression, Solution, SolverModel, Variable};

    pub struct GridProblem {
        vars: ProblemVariables,
        z: Vec<Vec<Variable>>,
        constraints: Vec<Constraint>,
        total_penalty: Expression,
        dim: u8,
        r_s: u8,
        r_p: u8,
    }

    impl GridProblem {
        /// Adds a new tower variable at the given point to the LP.
        fn add_tower_variable(&mut self, _tower: Point) -> Variable {
            let is_tower = self.vars.add(variable().binary());
            is_tower
        }

        /// Adds the penalty variable for point ij and tower kl to the LP.
        fn add_penalty_variables(&mut self) {
            for i in 0..(self.dim as usize) {
                for j in 0..(self.dim as usize) {
                    let p = Point::new(i as i32, j as i32);
                    let coverage = Point::points_within_radius(p, self.r_p, self.dim);
                    for point in coverage {
                        let k = point.x as usize;
                        let l = point.y as usize;

                        let p_ijkl = self.vars.add(variable().binary());
                        self.constraints.push(constraint!(p_ijkl <= self.z[i][j]));
                        self.constraints.push(constraint!(p_ijkl <= self.z[k][l]));
                        self.constraints.push(constraint!(p_ijkl >= self.z[i][j] + self.z[k][l] - 1));

                        self.total_penalty += p_ijkl;
                    }
                }
            }
        }

        /// Adds the city coverage constraints to the LP.
        fn add_city_constraints(&mut self, cities: HashSet<Point>) {
            for c in cities {
                let coverage = Point::points_within_radius(c, self.r_s, self.dim);
                let mut sum = Expression::with_capacity(coverage.len());
                for point in coverage {
                    sum += self.z[point.x as usize][point.y as usize];
                }
                self.constraints.push(constraint!(sum >= 1));
            }
        }


        /// Creates a new GridProblem instance.
        pub fn new(dim: u8, r_s: u8, r_p: u8, cities: HashSet<Point>) -> GridProblem {
            
            let mut lp = GridProblem {
                vars: variables![],
                constraints: vec![],
                z: vec![],
                dim,
                r_s,
                r_p,
                total_penalty: 2147483647.into(),
            };

            // add variables for each tower
            let dummy = lp.add_tower_variable(Point::new(-12345, -12345));
            lp.z = vec![vec![dummy; dim.into()]; dim.into()];
            for i in 0..dim {
                for j in 0..dim {
                    let tower = Point::new(i as i32, j as i32);
                    lp.z[i as usize][j as usize] = lp.add_tower_variable(tower);
                }
            }

            // add penalty variables
            lp.add_penalty_variables();

            // add city constraints
            lp.add_city_constraints(cities);


            // let variables: Vec<_> = products.into_iter().map(|p| pb.add(p)).collect();
            // let solution = pb.best_product_quantities();
            // let product_quantities: Vec<_> = variables.iter().map(|&v| solution.value(v)).collect();


            lp
        }

        /// Assumes everything (variables, constraints) has been added already
        fn solution(self) -> impl Solution {
            let mut v = self.vars.minimise(self.total_penalty).using(default_solver);
            for c in self.constraints {
                v = v.with(c);
            }
            v.solve().unwrap()
        }
        
        pub fn tower_solution(self) -> HashSet<Point> {
            const TOL: f64 = 1e-6;
            let solution = self.solution();
            let mut result = HashSet::new();
            for i in 0..(self.dim as usize) {
                for j in 0..(self.dim as usize) {
                    if (solution.value(self.z[i][j]) - 1.).abs() < TOL {
                        result.insert(Point::new(i as i32, j as i32));
                    }
                }
            }
            result
        }

    }
}
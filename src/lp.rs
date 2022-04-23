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



use crate::grid::Point;

use good_lp::{constraint, variable, variables,
    constraint::Constraint, variable::ProblemVariables, Expression, Solution, SolverModel, Variable};
use good_lp::{coin_cbc};
use std::collections::HashSet;


pub struct GridProblem {
    vars: ProblemVariables,
    z: Vec<Vec<Variable>>,
    constraints: Vec<Constraint>,
    total_penalty: Expression,
    dim: u8,
    r_s: u8,
    r_p: u8,
    max_time: u32, // in seconds
    console_log: u8,
    seed: u32,
}

impl GridProblem {
    /// Adds a new tower variable z_ij at the given point (i, j) to the LP.
    fn add_tower_variable(&mut self, tower: Point) -> Variable {
        let name = format!("z_{}_{}", tower.x, tower.y);
        let is_tower = self.vars.add(variable().binary().name(name));
        is_tower
    }

    /// Adds the penalty variable p_ijkl for point ij and tower kl to the LP.
    fn add_penalty_variables_ijkl(&mut self) {
        for i in 0..(self.dim as usize) {
            for j in 0..(self.dim as usize) {
                let p = Point::new(i as i32, j as i32);
                let coverage = Point::points_within_radius(p, self.r_p, self.dim);
                for point in coverage {
                    let k = point.x as usize;
                    let l = point.y as usize;

                    let name = format!("p_{}_{}_{}_{}", i, j, k, l);
                    let p_ijkl = self.vars.add(variable().binary().name(name));
                    self.constraints.push(constraint!(p_ijkl <= self.z[i][j]));
                    self.constraints.push(constraint!(p_ijkl <= self.z[k][l]));
                    self.constraints.push(constraint!(p_ijkl >= self.z[i][j] + self.z[k][l] - 1));

                    self.total_penalty += p_ijkl;
                }
            }
        }
    }

    /// Adds the penalty variables p_ij to the LP.
    fn add_penalty_variables(&mut self) {
        self.add_penalty_variables_ijkl(); //more optimal but hella slow. maybe consider for small inputs but that's it.
        // self.add_penalty_variables_leaky_relu(); // a balance between the two, for medium inputs. could use tuning.
        // self.add_penalty_variables_big_m(); // for large inputs; runs quickly and gives decent savings.
    }

    /// Adds the city coverage constraints to the LP.
    fn add_city_constraints(&mut self, cities: HashSet<Point>) {
        for c in cities {
            let coverage = Point::points_within_radius(c, self.r_s, self.dim);
            let mut sum = Expression::with_capacity(coverage.len());
            for point in coverage {
                sum.add_mul(1, self.z[point.x as usize][point.y as usize]);
            }
            self.constraints.push(sum.geq(1));
        }
    }

    pub fn new_randomized(dim: u8, r_s: u8, r_p: u8, cities: HashSet<Point>, max_time: u32, seed: u32) -> GridProblem {

        let mut lp = GridProblem {
            vars: variables![],
            constraints: vec![],
            z: vec![],
            dim,
            r_s,
            r_p,
            total_penalty: 0.into(),
            max_time,
            console_log: 0,
            seed
        };

        // add variables for each tower
        let dummy = lp.add_tower_variable(Point::new(-69420, -69420));
        lp.z = vec![vec![dummy; dim.into()]; dim.into()];
        for i in 0..dim {
            for j in 0..dim {
                let potential_tower = Point::new(i as i32, j as i32);
                lp.z[i as usize][j as usize] = lp.add_tower_variable(potential_tower);
                use rand::Rng;
                let r: i32 = rand::thread_rng().gen_range(-4..=4);
                lp.total_penalty += lp.z[i as usize][j as usize];
            }
        }

        // add city constraints
        lp.add_city_constraints(cities);

        lp
    }

    /// Creates and returns a new GridProblem LP.
    pub fn new(dim: u8, r_s: u8, r_p: u8, cities: HashSet<Point>, max_time: u32) -> GridProblem {
        let mut lp: GridProblem = GridProblem::new_randomized(dim, r_s, r_p, cities, max_time, 69420);
        lp.console_log = 1;
        lp.add_penalty_variables();

        lp
    }

    /// Assumes everything (variables, constraints) has been added already
    fn solution(self) -> impl Solution {
        let mut model = self.vars.minimise(self.total_penalty).using(coin_cbc);
        for c in self.constraints {
            model = model.with(c);
        }
        
        model.set_parameter("heur", "on");
        model.set_parameter("cuts", "on");
        // model.set_parameter("threads", "1"); //change to number of threads that you want
        // model.set_parameter("maxN", "300");
        // model.set_parameter("cutoff", "20");
        // // model.set_parameter("node", "fewest");
        // // model.set_parameter("multiple", "3");
        // model.set_parameter("sec", &self.max_time.to_string());
        
        model.set_parameter("randomSeed", &self.seed.to_string());
        model.set_parameter("randomC", &self.seed.to_string());
        // model.set_parameter("randomI", "on");
        model.set_parameter("log", &self.console_log.to_string()); // comment for less output
        model.solve().unwrap()
    }
    
    pub fn tower_solution(self) -> HashSet<Point> {
        const TOL: f64 = 1e-6;
        let d = self.dim as usize;
        let z = self.z.clone();
        let solution = self.solution();
        let mut result = HashSet::new();
        for i in 0..d {
            for j in 0..d {
                if (solution.value(z[i][j]) - 1.).abs() < TOL {
                    result.insert(Point::new(i as i32, j as i32));
                }
            }
        }
        result
    }

}

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

    /// Adds penalty variables using big-M constraints.
    fn add_penalty_variables_big_m(&mut self) {
        // p_ij for a given ij will be 0 if z_ij is 0 (tower not there)

        // and if z_ij is 1 (tower there) then p_ij will be sum of all the z_ij within penalty radius of ij.



        // how to formulate with lp constraints?
        // https://math.stackexchange.com/questions/2500415/how-to-write-if-else-statement-in-linear-programming


        // p_ij = z_ij * (sum(z_kl))
        //     = z_ij and z_k1l1 + z_ij and z_k1l2 + z_ij and z_k2l1 + z_ij and z_k2l2 + ...


        // if a > b then c = d else c = e.
        // a = z_ij, b = 0, c = p_ij, d = sum kl, e = 0


        // z_ij > 0 iff delta = 1

        // z_ij >= -M(1-delta)
        // z_ij <= M*delta
        // delta is binary

        // delta = 1 implies p_ij = sum(z_kl in penalty radius of ij)
        // delta = 0 implies p_ij = 0

        // sum kl - M(1 - delta) <= p_ij <= sum kl + M(1 - delta)
        // - M * delta <= p_ij <= M * delta

        const M: i32 = 10000000;
        let w_c: i32 = ((self.dim as f64) / 10.).floor() as i32; // number of towers until bad, ie cutoff
        let w_max: i32 = 2 * w_c; // absolute worst possible number of overlapping towers
        
        for i in 0..(self.dim as usize) {
            for j in 0..(self.dim as usize) {
                let p = Point::new(i as i32, j as i32);
                let coverage = Point::points_within_radius(p, self.r_p, self.dim);
                let z_ij = self.z[i][j];
                let delta = self.vars.add(variable().binary().name(format!("delta_{}_{}", i, j)));
                self.constraints.push(constraint!(z_ij >= -M * (1 - delta)));
                self.constraints.push(constraint!(z_ij <= M * delta));

                let mut w_ij = Expression::with_capacity(coverage.len());
                let p_ij = self.vars.add(variable().integer().name(format!("p_{}_{}", i, j)));
                for point in coverage {
                    w_ij.add_mul(1, self.z[point.x as usize][point.y as usize]);
                }

                // if want to impose hard cutoff
                self.constraints.push(constraint!(w_ij.clone() <= w_max));
                self.constraints.push(constraint!(w_ij.clone() - M * (1 - delta) <= p_ij));
                self.constraints.push(constraint!(p_ij <= w_ij.clone() + M * (1 - delta)));

                self.constraints.push(constraint!(-M * delta <= p_ij));
                self.constraints.push(constraint!(p_ij <= M * delta));

                self.total_penalty += p_ij;
            }
        }

    }

    fn penalty(x: i32) -> f64 {
        170. * (x as f64).exp().powf(0.17)
    }

    /// Returns (a, b) such that absolute deviation is minimized on x_min <= x < x_max for
    /// y = a * x + b and target P(x) := 170e^0.17x on integer inputs.
    fn penalty_lad_coeffs(x_min: i32, x_max: i32) -> (f64, f64) {            
        //target function y = P(w) := 170e^0.17w
        let y: Vec<f64> = (x_min..x_max).map(|x| Self::penalty(x)).collect();
        let x: Vec<f64> = (x_min..x_max).map(|x| Self::penalty(x)).collect();
        
        let mut v = variables![];
        let mut constraints = vec![];
        let mut objective = Expression::with_capacity(x.len());
        let a = v.add(variable());
        let b = v.add(variable());

        for i in 0..x.len() {
            let z_i = v.add(variable());
            constraints.push(constraint!(y[i] - (a + b * x[i]) <= z_i));
            constraints.push(constraint!((a + b * x[i]) - y[i] <= z_i));
            objective.add_mul(1, z_i);
        }

        let mut model = v.minimise(objective).using(coin_cbc);
        for c in constraints {
            model = model.with(c);
        }
        let solution = model.solve().unwrap();

        (solution.value(a), solution.value(b))
    }

    /// Returns (a, b) such that ax+b is the line connecting 
    /// (x_min, p(x_min)) and (x_max, p(x_max)).
    fn penalty_simple_coeffs(x_min: i32, x_max: i32) -> (f64, f64) {
        let y_min = Self::penalty(x_min);
        let y_max = Self::penalty(x_max);

        let a = ((y_max - y_min) as f64) / ((x_max - x_min) as f64);
        let b = y_min - a * (x_min as f64);
        (a, b)
    }

    /// Adds penalty variables using a leaky-relu function.
    fn add_penalty_variables_leaky_relu(&mut self) {
        // !!!!!!!!!!
        // LEAKY RELU
        // !!!!!!!!!!

        // if (a > b and x > y) then c = d else c = e.

        // alternatively
        // if (a>b) then
        //     if (x > y) then
        //         s = d
        //     else
        //         s = e
            
        //     c = s

        // else
        //     c = f

        // cutoff = w_c = number of towers until bad, say 5
        // m1 = slope before cutoff, say 1
        // m2 = slope after cutoff, say 10
        // s = slack variable for assignment

        // w_ij = number of overlapping towers = sum_kl

        // a = z_ij, b = 0, x = w_ij, y = w_c, c = p_ij, d = m1 * s, e = m2 * s, f = 0


        // z_ij > 0 iff delta = 1
        //     z_ij >= -M(1-delta)
        //     z_ij <= M*delta
        //     delta is binary


        //     w_ij > w_c iff eps = 1
        //         w_ij >= w_c - M(1-eps)
        //         w_ij <= w_c + M*eps
        //         eps is binary

        //     eps = 1 implies s = m1 * w_ij
        //     eps = 0 implies s = m2 * w_ij

        //         m1 * w_ij - M (1 - eps) <= s <= m1 * w_ij + M (1 - eps)
        //         m2 * w_ij - M * eps <= s <= m2 * w_ij + M * eps



        // delta = 1 implies p_ij = s
        // delta = 0 implies p_ij = 0

        //     s - M(1 - delta) <= p_ij <= s + M(1 - delta)
        //     - M * delta <= p_ij <= M * delta

        // maximum penalty constraint : w_ij <= w_max

        const M: i32 = 10000000;
        let w_c: i32 = ((self.dim as f64) / 10.).floor() as i32; // number of towers until bad, ie cutoff
        let w_max: i32 = 2 * w_c; // absolute worst possible number of overlapping towers
        
        let (m1, b1) = Self::penalty_simple_coeffs(0, w_c);
        let (m2, b2) = Self::penalty_simple_coeffs(w_c, w_max);
        println!("m1 = {:?}, b1 = {:?}, m2 = {:?}, b2 = {:?}", m1, b1, m2, b2);
        // let m1 = 1; // slope before cutoff
        // let b1 = -1; // intercept before cutoff
        // let m2 = 10; // slope after cutoff
        // let b2: i32 = m1 * w_c + b1 - m2 * w_c; // intercept after cutoff, to make function continuous

        for i in 0..(self.dim as usize) {
            for j in 0..(self.dim as usize) {
                let p = Point::new(i as i32, j as i32);
                let coverage = Point::points_within_radius(p, self.r_p, self.dim);
                let p_ij = self.vars.add(variable().integer().name(format!("p_{}_{}", i, j)));
                let z_ij = self.z[i][j];
                let delta = self.vars.add(variable().binary().name(format!("delta_{}_{}", i, j)));
                self.constraints.push(constraint!(z_ij >= -M * (1 - delta)));
                self.constraints.push(constraint!(z_ij <= M * delta));

                let mut w_ij = Expression::with_capacity(coverage.len());
                
                for point in coverage {
                    let z_xy = self.z[point.x as usize][point.y as usize];
                    w_ij.add_mul(1, z_xy);
                }

                // self.constraints.push(constraint!(w_ij.clone() <= w_max));

                let eps = self.vars.add(variable().binary().name(format!("eps_{}_{}", i, j)));
                self.constraints.push(constraint!(w_ij.clone() >= w_c - M * (1 - eps)));
                self.constraints.push(constraint!(w_ij.clone() <= w_c + M * eps));

                let s = self.vars.add(variable().integer().name(format!("s_{}_{}", i, j)));
                // let d = m1 * w_ij.clone() + b1;
                // let e = m2 * w_ij.clone() + b2;

                self.constraints.push(constraint!(m1 * w_ij.clone() + b1 - M * (1 - eps) <= s));
                self.constraints.push(constraint!(s <= m1 * w_ij.clone() + b1 + M * (1 - eps)));
                self.constraints.push(constraint!(m2 * w_ij.clone() + b2 - M * eps <= s));
                self.constraints.push(constraint!(s <= m2 * w_ij.clone() + b2 + M * eps));

                self.constraints.push(constraint!(s - M * (1 - delta) <= p_ij));
                self.constraints.push(constraint!(p_ij <= s + M * (1 - delta)));

                self.constraints.push(constraint!(-M * delta <= p_ij));
                self.constraints.push(constraint!(p_ij <= M * delta));

                self.total_penalty += p_ij;
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


    /// Creates and returns a new GridProblem LP.
    pub fn new(dim: u8, r_s: u8, r_p: u8, cities: HashSet<Point>, max_time: u32) -> GridProblem {
        
        let mut lp = GridProblem {
            vars: variables![],
            constraints: vec![],
            z: vec![],
            dim,
            r_s,
            r_p,
            total_penalty: 2147483647.into(),
            max_time
        };

        // add variables for each tower
        let dummy = lp.add_tower_variable(Point::new(-69420, -69420));
        lp.z = vec![vec![dummy; dim.into()]; dim.into()];
        for i in 0..dim {
            for j in 0..dim {
                let potential_tower = Point::new(i as i32, j as i32);
                lp.z[i as usize][j as usize] = lp.add_tower_variable(potential_tower);
            }
        }

        // add penalty variables
        lp.add_penalty_variables();
        // add city constraints
        lp.add_city_constraints(cities);

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
        model.set_parameter("threads", "1"); //change to number of threads that you want
        // model.set_parameter("maxN", "300");
        // model.set_parameter("cutoff", "20");
        // // model.set_parameter("node", "fewest");
        // // model.set_parameter("multiple", "3");
        // model.set_parameter("sec", &self.max_time.to_string());
        // model.set_parameter("randomSeed", &69420.to_string());
        // model.set_parameter("randomC", &69420.to_string());
        // model.set_parameter("log", &3.to_string()); // comment for less output
        model.solve().unwrap()
    }
    
    pub fn tower_solution(self) -> HashSet<Point> {
        const TOL: f64 = 1e-6;
        let d = self.dim as usize;
        let z = self.z.clone();
        let solution = self.solution();
        let mut result = HashSet::new();
        for i in 0..(d) {
            for j in 0..(d) {
                if (solution.value(z[i][j]) - 1.).abs() < TOL {
                    result.insert(Point::new(i as i32, j as i32));
                }
            }
        }
        result
    }

}

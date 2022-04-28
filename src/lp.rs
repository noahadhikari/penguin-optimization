use std::collections::HashSet;

use good_lp::constraint::Constraint;
use good_lp::variable::ProblemVariables;
use good_lp::{coin_cbc, constraint, variable, variables, Expression, Solution, SolverModel, Variable};

use crate::point::Point;

/// Idea: Because penalty is monotonic ish, can try to minimize a linear penalty
/// to use LP.
///
/// let t_{ij} = {0, 1} correspond to whether or not a tower is placed at (i, j)
///
/// for each city, ensure that at least one tower is covering it.
/// i.e. for each c_ij, sum of t_{ij} in coverage(c_ij) >= 1
///
/// for each point in t, want to calculate penalty for that point, but only if
/// the tower is actually there. let p_{ij,kl} = penalty at position ij for
/// position kl only if tower t_ij is present and t_kl exists for all
///     kl in the penalty coverage of ij.
/// p_ijkl = t_kl if t_ij else 0
/// -> t_ij AND t_kl
/// -> p_ijkl <= t_kl, p_ijkl <= t_ij, p_ijkl >= t_ij + t_kl - 1.
///
///
/// all variables are binary except the total penalty (maybe unnecessary, but P
/// = sum_ij p_ij). minimize sum of p_ij (== P).
///
/// ------------------------------
///
/// total number of variables is on the order of R^2 * d^2.

pub struct GridProblem {
	vars:          ProblemVariables,
	t:             Vec<Vec<Variable>>,
	constraints:   Vec<Constraint>,
	total_penalty: Expression,
	dim:           u8,
	r_s:           u8,
	r_p:           u8,
	max_time:      u32, // in seconds
	console_log:   u8,
	seed:          u32,
}

impl GridProblem {
	/// Adds a new tower variable t_ij at the given point (i, j) to the LP.
	fn add_tower_variable(&mut self, _tower: Point) -> Variable {
		// let name = format!("t_{}_{}", tower.x, tower.y);
		let is_tower = self.vars.add(variable().binary()); //.name(name));
		is_tower
	}

	/// Adds the penalty variable p_ijkl for point ij and tower kl to the LP.
	fn add_penalty_variables(&mut self) {
		for i in 0..(self.dim as usize) {
			for j in 0..(self.dim as usize) {
				let p = Point::new(i as i32, j as i32);
				let coverage = Point::points_within_radius(p, self.r_p, self.dim).unwrap();
				for point in coverage {
					let k = point.x as usize;
					let l = point.y as usize;

					// let name = format!("p_{}_{}_{}_{}", i, j, k, l);
					let p_ijkl = self.vars.add(variable().binary()); //.name(name));
					self.constraints.push(constraint!(p_ijkl <= self.t[i][j]));
					self.constraints.push(constraint!(p_ijkl <= self.t[k][l]));
					self
						.constraints
						.push(constraint!(p_ijkl >= self.t[i][j] + self.t[k][l] - 1));

					self.total_penalty += p_ijkl;
				}
			}
		}
	}

	/// Adds the city coverage constraints to the LP.
	fn add_city_constraints(&mut self, cities: HashSet<Point>) {
		for c in cities {
			let coverage = Point::points_within_radius(c, self.r_s, self.dim).unwrap();
			let mut sum = Expression::with_capacity(coverage.len());
			for point in coverage {
				sum.add_mul(1, self.t[point.x as usize][point.y as usize]);
			}
			self.constraints.push(sum.geq(1));
		}
	}

	/// Creates a new grid for randomization solving.
	pub fn new_randomized(dim: u8, r_s: u8, r_p: u8, cities: HashSet<Point>, max_time: u32, seed: u32) -> Self {
		let mut lp = GridProblem {
			vars: variables![],
			constraints: vec![],
			t: vec![],
			dim,
			r_s,
			r_p,
			total_penalty: 0.into(),
			max_time,
			console_log: 0,
			seed,
		};

		// add variables for each tower
		let dummy = lp.add_tower_variable(Point::new(-69420, -69420));
		lp.t = vec![vec![dummy; dim.into()]; dim.into()];
		for i in 0..dim {
			for j in 0..dim {
				let potential_tower = Point::new(i as i32, j as i32);
				lp.t[i as usize][j as usize] = lp.add_tower_variable(potential_tower);
				lp.total_penalty += lp.t[i as usize][j as usize];
			}
		}

		// ignores penalty constraints for randomization

		// add city constraints
		lp.add_city_constraints(cities);

		lp
	}

	/// Creates and returns a new GridProblem LP.
	pub fn new(dim: u8, r_s: u8, r_p: u8, cities: HashSet<Point>, max_time: u32) -> Self {
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
		// model.set_parameter("threads", "1"); //change to number of threads that you
		// want model.set_parameter("maxN", "300");
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
		let t = (&self.t).clone();
		let solution = self.solution();
		let mut result = HashSet::new();
		for i in 0..d {
			for j in 0..d {
				if (solution.value(t[i][j]) - 1.).abs() < TOL {
					result.insert(Point::new(i as i32, j as i32));
				}
			}
		}
		result
	}
}

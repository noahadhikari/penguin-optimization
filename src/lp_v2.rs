use std::collections::HashSet;

use good_lp::constraint::Constraint;
use good_lp::variable::ProblemVariables;
use good_lp::{constraint, highs, variable, variables, Expression, Solution, SolverModel, Variable};

use crate::point::Point;

pub struct GridProblem {
	vars:          ProblemVariables,
	t:             Vec<Vec<Variable>>,
	w:             Vec<Vec<Expression>>,
	constraints:   Vec<Constraint>,
	total_penalty: Expression,
	dim:           usize,
	r_s:           u8,
	r_p:           u8,
	max_time:      u32, // in seconds
	console_log:   u8,
}

impl GridProblem {

	/// Creates and returns a new GridProblem LP.
	pub fn new(dim: usize, r_s: u8, r_p: u8, cities: Vec<Point>, max_time: u32) -> Self {
		// TODO add max time
		let mut pb = GridProblem {
			vars: variables![],
			constraints: Vec::new(),
			t: vec![vec![]; dim],
			w: vec![vec![]; dim],
			dim,
			r_s,
			r_p,
			total_penalty: 0.into(),
			max_time,
			console_log: 0,
		};

		// Fill the vector of vectors with dummy variables/expressions
		let dummy = pb.vars.add(variable().binary());
		pb.t = vec![vec![dummy; dim]; dim];
		pb.w = vec![vec![dummy + 1; dim]; dim];

		pb.add_all_tower_variables();
		pb.add_all_penalty_variables(); // add penalty variables after tower variables
		pb.add_city_constraints(cities);
		pb.add_objective_function();

		pb
	}


	/// Adds a new tower variable t_ij at the given point (i, j) to the LP.
	fn add_tower_variable(&mut self, _tower: Point) -> Variable {
		// let name = format!("t_{}_{}", tower.x, tower.y);
		let is_tower = self.vars.add(variable().binary());
		is_tower
	}

	// This is wack to get around not letting me multiply current tower to w_ij
	/// Adds the penalty variable w_ij at the given point (i, j) to the LP.
	/// Represents the w_ij penalty if a tower existed at that point.
	fn add_penalty_expression(&mut self, tower: Point) -> Expression {
		// All possible towers around it
		let coverage = Point::points_within_radius(tower, self.r_p, self.dim as u8).unwrap();
		let slack_var = self.vars.add(variable().integer());
		let mut sum = Expression::with_capacity(coverage.len());

		// Relationship between w_ij and t_ij
		for point in coverage {
			sum += self.t[point.x as usize][point.y as usize];
		}

		// Let slack bar equal the sum of all t_ij but with constraints!
		for point in coverage {
			self.constraints.push(constraint!(slack_var <= self.t[point.x as usize][point.y as usize]));
		}

		self.constraints.push(constraint!(slack_var >= sum));

		let w_ij = self.t[tower.x as usize][tower.y as usize] * slack_var;

		w_ij
	}

	/// Adds all possible tower variables to the LP.
	fn add_all_tower_variables(&mut self) {
		for i in 0..self.dim {
			for j in 0..self.dim {
				let tower = Point::new(i as i32, j as i32);
				self.t[i][j] = self.add_tower_variable(tower);
			}
		}
	}

	/// Adds all the penalty variables and their relationship to tower variables
	/// to the LP. (penalty variables are added to the LP after tower variables)
	fn add_all_penalty_variables(&mut self) {
		for i in 0..self.dim {
			for j in 0..self.dim {
				let tower = Point::new(i as i32, j as i32);
				self.w[i][j] = self.add_penalty_expression(tower);
			}
		}
	}

	/// Adds the city coverage constraints to the LP.
	fn add_city_constraints(&mut self, cities: Vec<Point>) {
		for c in cities {
			let coverage = Point::points_within_radius(c, self.r_s, self.dim as u8).unwrap();
			let mut sum = Expression::with_capacity(coverage.len());
			for point in coverage {
				sum += self.t[point.x as usize][point.y as usize];
			}
			self.constraints.push(sum.geq(1));
		}
	}

	// TODO: Make this piecewise linear
	
	/// Defines the objective function of the LP.
	/// The objective function is the sum of all the penalty variables.
	fn add_objective_function(&mut self) {
		let mut sum = Expression::with_capacity((self.dim * self.dim) as usize);
		for i in 0..(self.dim as usize) {
			for j in 0..(self.dim as usize) {
				sum += self.t[i][j] *  self.w[i][j];
			}
		}
		self.total_penalty = sum;
	}

	fn solution(self) -> impl Solution {
		let mut model = self.vars.minimise(self.total_penalty).using(highs);

		for c in self.constraints {
			model = model.with(c);
		}

		model.set_verbose(true);

		model.solve().unwrap()
	}

	pub fn into_tower_solution(self) -> HashSet<Point> {
		const TOL: f64 = 1e-6;

		let dim = self.dim;

		let t = (&self.t).clone();
		let solution = self.solution();
		let mut result = HashSet::new();

		for i in 0..dim {
			for j in 0..dim {
				if (solution.value(t[i][j]) - 1.).abs() < TOL {
					result.insert(Point::new(i as i32, j as i32));
				}
			}
		}
		result
	}
}

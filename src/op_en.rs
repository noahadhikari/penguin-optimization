use std::collections::HashSet;

use optimization_engine::alm::*;
use optimization_engine::core::constraints::*;
use optimization_engine::core::panoc::*;
use optimization_engine::{matrix_operations, SolverError};

use crate::grid::Grid;
use crate::point::Point;

// Following https://github.com/alphaville/optimization-engine/blob/master/examples/alm_pm.rs

// TODO: have everything in project use a usize unless necessary

const TOLERANCE: f64 = 1e-14;
const MAX_ITERATIONS: usize = 100;

// t[dim * i + j] is the tower variable t_ij (same for w)

// Some vectors are the grid in row major order to
// make it easier to dot product with the kernel
// note: kernel might be an abuse of the term, they're filters
pub struct OpEnProblem {
	dim: usize,
	// phi[i][j] is the kernel phi_ij
	phi: Vec<Vec<Vec<f64>>>,

	// c[i] is the kernel for city i
	c: Vec<Vec<f64>>,
}

impl OpEnProblem {
	/// Creates and returns a new OpEnProblem from a grid.
	pub fn new(grid: &Grid) -> Self {
		let num_cities = grid.get_cities_ref().len();
		let dim = grid.dimension() as usize;

		let phi = vec![vec![vec![0.0; dim * dim]; dim]; dim];
		let c = vec![vec![0.0; dim]; num_cities];

		let mut oep = OpEnProblem { dim, phi, c };

		for (i, (city, _)) in grid.get_cities_ref().iter().enumerate() {
			oep.create_city_kernel(city, i, grid.service_radius() as usize);
		}

		oep.create_phi_kernels(grid.penalty_radius() as usize);

		oep
	}

	/// Solves the OpEn problem, returning a set of towers.
	pub fn into_tower_solution(&mut self) -> HashSet<Point> {
		const TOL: f64 = 1e-6;

		let t = self.solve();
		let mut result = HashSet::new();

		for i in 0..self.dim {
			for j in 0..self.dim {
				if (t[i * self.dim + j] - 1.).abs() < TOL {
					result.insert(Point::new(i as i32, j as i32));
				}
			}
		}
		result
	}

	/// Solves the OpEn problem (using the Augmented Lagrangian, check paper)
	fn solve(&mut self) -> Vec<f64> {
		let tolerance = 1e-5;
		// nx (nu in the documentation) is the number of variables
		let nx = self.dim * self.dim;
		// n1 is the output dimension of F1 (num of cities)
		let n1 = self.c.len();
		// n2 is the output dimension of F2 (which we don't have)
		let n2 = 0;
		let lbfgs_mem = 3;
		let panoc_cache = PANOCCache::new(nx, tolerance, lbfgs_mem);
		let mut alm_cache = AlmCache::new(panoc_cache, n1, n2);

		// CONSTRAINTS (https://docs.rs/optimization_engine/0.7.4/optimization_engine/constraints/index.html)
		// Set C is used to define the constraints, ie F1(u) \in C
		// We want 1 \le F1 \le \infty (ie at least one tower covering a city)
		// (elementwise)
		let set_c = Rectangle::new(Some(&[1.0]), None);

		// Bounds are the constraints for our inputs (u in the docs, here t)
		// We want this to be a finite set, each t_i can either be 0 or 1
		// let bounds = self.get_tower_constraints();

		// try to relax bounds
		let bounds = Ball2::new(None, 10.0);


		// Compact, convex set of Lagrange multipliers, which needs to be a compact
		// subset of (the convex conjugate of the convex set C) hopefully the example
		// one is enough bc I'm tired of math
		let set_y = Ball2::new(None, 1e12);

		// Need to define necessary functions in here since they need the dynamic
		// environment
		let f = |t: &[f64], cost: &mut f64| self.cost(t, cost);
		let df = |t: &[f64], grad: &mut [f64]| self.grad_cost(t, grad);
		let f1 = |t: &[f64], out: &mut [f64]| self.mapping_f1(t, out);
		let f1_jacobian_product = |t: &[f64], d: &[f64], out: &mut [f64]| self.jacobian_mapping_f1_trans(t, d, out);

		let factory = AlmFactory::new(
			f,
			df,
			Some(f1),
			Some(f1_jacobian_product),
			NO_MAPPING,
			NO_JACOBIAN_MAPPING,
			Some(set_c),
			0, // n_2 is the output dimensionality
		);

		// Now that the factory made the needed functions
		// (https://docs.rs/optimization_engine/0.7.4/optimization_engine/alm/struct.AlmFactory.html)
		// we can create the alm problem

		let alm_problem = AlmProblem::new(
			bounds,
			Some(set_c),
			Some(set_y),
			|u: &[f64], xi: &[f64], cost: &mut f64| -> Result<(), SolverError> { factory.psi(u, xi, cost) },
			|u: &[f64], xi: &[f64], grad: &mut [f64]| -> Result<(), SolverError> { factory.d_psi(u, xi, grad) },
			Some(f1),
			NO_MAPPING,
			n1,
			n2,
		);

		let mut alm_optimizer = AlmOptimizer::new(&mut alm_cache, alm_problem)
			.with_delta_tolerance(1e-5)
			.with_max_outer_iterations(200);
		// .with_epsilon_tolerance(1e-6)
		// .with_initial_inner_tolerance(1e-2)
		// .with_inner_tolerance_update_factor(0.5)
		// .with_initial_penalty(100.0)
		// .with_penalty_update_factor(1.05)
		// .with_sufficient_decrease_coefficient(0.2)
		// .with_initial_lagrange_multipliers(&vec![5.0; n1]);

		let mut t = vec![0.75; self.dim * self.dim];

		let solver_result = alm_optimizer.solve(&mut t);
		let r = solver_result.unwrap();

		println!("\n\nSolver result : {:#.7?}\n", r);

		for i in 0..self.dim {
			for j in 0..self.dim {
				print!("{:.2} ", t[i * self.dim + j]);
			}
			println!();
		}
		println!("-------------------------------------------------------");
		t
	}

	// -------- CONSTRAINTS --------
	fn get_tower_constraints(&self) -> CartesianProduct {
		let mut bounds = CartesianProduct::new_with_capacity(self.dim * self.dim);
		for i in 1..(self.dim * self.dim + 1) {
			bounds = bounds.add_constraint(i, FiniteSet::new(&[&[0.0], &[1.0]]));
		}
		bounds
	}

	// -------- HELPER FUNCTIONS FOR AUGMENTED LAGRANGIAN METHOD --------

	/// Function F_1 in the paper
	fn mapping_f1(&self, t: &[f64], f1: &mut [f64]) -> Result<(), SolverError> {
		let mut out_vec: Vec<_> = vec![0.0; self.c.len()];

		for i in 0..self.c.len() {
			out_vec[i] = matrix_operations::inner_product(self.c[i].as_slice(), t);
			println!("{:.2}", out_vec[i]);
		}

		f1.copy_from_slice(out_vec.as_slice());
		Ok(())
	}

	/// Jacobian of F_1 evaluated at t, transposed and multiplied by d
	fn jacobian_mapping_f1_trans(&self, _t: &[f64], d: &[f64], out: &mut [f64]) -> Result<(), SolverError> {
		// I'm too tired to iterate through i's and j's so forgive me if I don't
		// Maybe they should all be like this? idk what would be clearer?

		// TODO: check my math

		for row in 0..(self.dim * self.dim) {
			let mut out_row = 0.0;
			for col in 0..self.c.len() {
				// This is adding d_col * partial of the dot product between t and c with
				// respect to t_row
				out_row += self.c[col][row] * d[col];
			}
			out[row] = out_row;
		}

		Ok(())
	}

	// -------- COST FUNCTIONS --------

	// TODO: add unit tests for these math functions
	// Cost and grad functions as defined here:
	// https://alphaville.github.io/optimization-engine/docs/openrust-basic

	fn cost(&self, t: &[f64], cost: &mut f64) -> Result<(), SolverError> {
		// C = sum_i sum_j 170 * t_ij * e ^ {0.17 * w_ij}}
		let w = self.get_w(t);

		*cost = 0.0;

		for i in 0..self.dim {
			for j in 0..self.dim {
				*cost += 170.0 * t[i * self.dim + j] * (w[i * self.dim + j] * 0.17).exp();
			}
		}

		Ok(())
	}

	// TODO: doesn't need to be an instance method
	fn get_w(&self, t: &[f64]) -> Vec<f64> {
		let mut w = vec![0.0; self.dim * self.dim];

		for i in 0..self.dim {
			for j in 0..self.dim {
				// Calculate the number of towers within the penalty radius
				w[i * self.dim + j] = matrix_operations::inner_product(self.phi[i][j].as_slice(), t);
			}
		}

		w
	}

	fn grad_cost(&self, t: &[f64], grad: &mut [f64]) -> Result<(), SolverError> {
		// Check paper for details on the gradient
		let w = self.get_w(t);

		for i in 0..self.dim {
			for j in 0..self.dim {
				// Calculate the grad with respect to t_ij
				let mut grad_ij = 0.0;

				for k in 0..self.dim {
					for l in 0..self.dim {
						grad_ij += 170.0 * t[k * self.dim + l] * (w[k * self.dim + l] * 0.17).exp() * self.grad_w((k, l), (i, j));
					}
				}

				grad_ij += 170.0 * (0.17 * w[i * self.dim + j]).exp();
				grad[i * self.dim + j] = grad_ij;
			}
		}


		Ok(())
	}

	/// Calculates the partial of w_kl with respect to t_ij
	fn grad_w(&self, of: (usize, usize), wrt: (usize, usize)) -> f64 {
		// Check paper for why this works
		let (k, l) = of;
		let (i, j) = wrt;

		self.phi[k][l][i * self.dim + j]
	}

	// -------- KERNEL HELPERS --------

	fn create_city_kernel(&mut self, city: &Point, city_id: usize, service_radius: usize) {
		let mut kernel = vec![0.0; self.dim * self.dim];

		// Get all points that would cover the city
		let satisfied_points = Point::points_within_radius(*city, service_radius as u8, self.dim as u8).unwrap();

		for point in satisfied_points {
			kernel[self.dim * point.x as usize + point.y as usize] = 1.0;
		}

		self.c[city_id] = kernel;
	}

	// TODO: persist the phi kernels
	fn create_phi_kernels(&mut self, penalty_radius: usize) {
		for i in 0..self.dim {
			for j in 0..self.dim {
				// Create the kernel for phi_ij
				// Get all points that would be counted as a penalty for a tower at (i, j)
				let penalty_points =
					Point::points_within_radius(Point::new(i as i32, j as i32), penalty_radius as u8, self.dim as u8).unwrap();

				// change phi_ij
				for point in penalty_points {
					self.phi[i][j][self.dim * point.x as usize + point.y as usize] = 1.0;
				}
			}
		}
	}
}

// Used to ignore unused code warnings.
#![allow(dead_code)]

mod grid;
mod lp;
use grid::Grid;

use std::fmt::Error;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Write;
use std::io::{self, BufReader};
use stopwatch::Stopwatch;

fn solve_all_inputs() {
    const CUTOFF_TIME: u32 = 500000; //max time in seconds

    let paths = fs::read_dir("./inputs/small").unwrap();

    for path in paths {
        let real_path = path.unwrap().path();
        // ie: 001
        let test_number = real_path.file_stem().unwrap().to_str().unwrap();
        let input_path = real_path.to_str().unwrap();
        let output_path = "./outputs/".to_string() + "small/" + test_number + ".out";

        let mut grid = get_grid(input_path).unwrap();

        grid.lp_solve(CUTOFF_TIME);

        write_sol(&grid, &output_path);
    }
}

fn solve_one_input() {
    const INPUT_PATH: &str = "./inputs/test/tiny.in";
    const OUTPUT_PATH: &str = "./outputs/test/tiny.out";
    let mut grid = get_grid(INPUT_PATH).unwrap();
    const CUTOFF_TIME: u32 = 3600; //max time in seconds
    grid.lp_solve(CUTOFF_TIME);

    write_sol(&grid, OUTPUT_PATH);
    // println!("Valid: {}", grid.is_valid());
    println!("{}", grid);
}

fn solve_all_randomized() {
	let paths = fs::read_dir("./inputs/medium").unwrap();

	// Will find a better way for this
	let mut i = 1;
	for path in paths {

		// There's probably a much better way to do this
		match i {
			1..=10 => {
				// Uncomment for 11-20
				// i += 1;
				// continue
			}, 
			11..=20 => {
				// Comment out for 11-20
				continue;
			}
			_ => return,
		}
		
		let real_path = path.unwrap().path();
		let test_number = real_path.file_stem().unwrap().to_str().unwrap(); // ie: 001
		let input_path = real_path.to_str().unwrap();
		let output_path = "./outputs/".to_string() + "medium/" + test_number + ".out";
		solve_one_randomized(input_path, &output_path, 300);
		
		i += 1;
	}

}

fn solve_one_randomized(input_path: &str, output_path: &str, secs_per_input: u64) {
    // const INPUT_PATH: &str = "./inputs/medium/001.in";
    // const OUTPUT_PATH: &str = "./outputs/medium/001.out";
    const CUTOFF_TIME: u32 = 60; //max time in seconds
    const ITERATIONS: u32 = 10000;

    let mut best_penalty_so_far = f64::INFINITY;
    let mut best_grid_so_far = Grid::new(0, 0, 0);
		let sw = Stopwatch::start_new();
		// For every file:			
		while sw.elapsed().as_secs() < secs_per_input { // 5 mins
			let mut grid = get_grid(input_path).unwrap(); // Need a way to move this out
			let p = grid.random_lp_solve(CUTOFF_TIME);
			// println!("{} penalty: {}", i, p);
			if p < best_penalty_so_far {
					best_penalty_so_far = best_penalty_so_far.min(p);
					best_grid_so_far = grid;
			}

			let time = sw.elapsed().as_secs();
			if sw.elapsed().as_secs() % 10 == 0 {
				println!("{} secs passed. Best so far: {}", time, best_penalty_so_far);
			}
		}
    println!("Best: {}", best_penalty_so_far);
    println!("Valid: {}", best_grid_so_far.is_valid());
    write_sol(&best_grid_so_far, output_path);
}

fn main() {
    // solve_all_inputs();
    // solve_one_input();
    // solve_one_randomized();
		solve_all_randomized();
}

// Algorithms

/// Greedy algorithm for benchmarking.
/// Places towers at all city locations that haven't been covered
fn place_at_cities(grid: &mut Grid) {
    let cities = grid.get_cities().clone();
    let city_points = cities.keys();
    println!("{:?}", city_points);
    for point in city_points {
        let covered = grid.get_cities().get(point).unwrap();
        if covered.len() > 0 {
            continue;
        }
        grid.add_tower(point.get_x(), point.get_y());
    }
}

/// Returns the grid created from the passed in input file.
fn get_grid(path: &str) -> io::Result<Grid> {
    let mut g = Grid::new(0, 0, 0);

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut i: i32 = 0;
    let mut num_cities: i32 = -1;
    for line in reader.lines() {
        if let Ok(l) = line {
            let vec: Vec<&str> = l.split_whitespace().collect();
            let first_val: &str = vec.get(0).unwrap();
            if first_val.eq("#") {
                continue;
            }
            match i {
                0 => num_cities = first_val.parse::<i32>().unwrap(),
                1 => g.set_dimension(first_val.parse::<u8>().unwrap()),
                2 => g.set_service_radius(first_val.parse::<u8>().unwrap()),
                3 => g.set_penalty_radius(first_val.parse::<u8>().unwrap()),
                _ => {
                    if (4..(4 + num_cities)).contains(&i) {
                        let x = first_val.parse::<i32>().unwrap();
                        let y = vec.get(1).unwrap().parse::<i32>().unwrap();
                        g.add_city(x, y);
                    }
                    // else {
                    //     println!("Past all cities");
                    // }
                }
            }
            i += 1;
        }
    }
    Ok(g)
}

fn write_sol(grid: &Grid, path: &str) {
    let data = grid.output();
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Unable to open file");
    f.write_all(data.as_bytes()).expect("Unable to write data");
}

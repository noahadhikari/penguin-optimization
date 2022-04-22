// Used to ignore unused code warnings.
#![allow(dead_code)]


mod grid;
use grid::Grid;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Write;
use std::io::{self, BufReader};

fn main() {
    const INPUT_PATH: &str = "./inputs/small.in";
    const OUTPUT_PATH: &str = "./outputs/small.out";
    let mut grid = get_grid(INPUT_PATH).unwrap();

    // place_at_cities(&mut grid);

    const CUTOFF_TIME: u32 = 300; //max time in seconds
    grid.lp_solve(CUTOFF_TIME);

    write_sol(&grid, OUTPUT_PATH);
    println!("Valid: {}", grid.is_valid());
    println!("{}", grid);
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
            let vec: Vec<&str> = l.split(' ').collect();
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

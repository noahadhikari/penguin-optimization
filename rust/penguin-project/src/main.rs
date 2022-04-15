// Used to ignore unused code warnings.
#![allow(dead_code)]

mod grid;

use grid::Grid;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

fn main() {
    const PATH: &str = "./inputs/tiny.in";
    let mut grid = get_grid(PATH).unwrap();
    grid.add_tower(0, 0);
    grid.add_tower(0, 5);
    grid.add_tower(3, 4);
    grid.add_tower(4, 3);
    // grid.add_tower(1, 1);
    // grid.add_tower(2, 3);
    // grid.add_tower(3, 5);
    // grid.add_tower(1, 2);
    println!("{:#?}", grid);
    println!("{}", grid);
}

/// Returns the grid created from the passed in input file.
fn get_grid(path: &str) -> io::Result<Grid> {
    let mut g = Grid::new(0, 0, 0);

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut i = 0;
    for line in reader.lines() {
        if let Ok(l) = line {
            let vec: Vec<&str> = l.split(' ').collect();
            let first_val: &str = vec.get(0).unwrap();
            if first_val.eq("#") {
                continue;
            }
            match i {
                0 => (), //println!("Number of cities: {}", first_val),
                1 => g.set_dimension(first_val.parse::<u8>().unwrap()),
                2 => g.set_service_radius(first_val.parse::<u8>().unwrap()),
                3 => g.set_penalty_radius(first_val.parse::<u8>().unwrap()),
                _ => {
                    // TODO: Fix this so no error will occur when there are a lot of newlines at the end of fle
                    let x = first_val.parse::<isize>().unwrap();
                    let y = vec.get(1).unwrap().parse::<isize>().unwrap();
                    g.add_city(x, y);
                }
            }
            i += 1;
        }
    }
    Ok(g)
}

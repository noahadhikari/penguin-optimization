mod grid;

fn main() {
    let mut g = grid::Grid::new(3, 3, 3);
    g.add_city(1, 2);
    g.add_tower(2, 2);
    g.add_city(2, 2);
    println!("{:?}", g);
}

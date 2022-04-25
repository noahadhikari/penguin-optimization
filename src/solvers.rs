use crate::grid::Grid;



// -- Naive Greedy --
/// Greedy algorithm for benchmarking.
/// Places towers at all city locations that haven't been covered
pub fn benchmark_greedy(grid: &Grid, sol: &mut Grid) {
	let cities = grid.get_cities().clone();
	let city_points = cities.keys();

	for city in city_points {
		let covered = grid.get_cities().get(city).unwrap();
		if covered.len() > 0 {
			continue;
		}
		sol.add_tower(city.get_x(), city.get_y());
	}
}
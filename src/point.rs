use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

lazy_static! {
    static ref PEN_S: HashMap<Point, HashSet<Point>> = preprocess::load("small", "penalty");
    static ref PEN_M: HashMap<Point, HashSet<Point>> = preprocess::load("medium", "penalty");
    static ref PEN_L: HashMap<Point, HashSet<Point>> = preprocess::load("large", "penalty");

    static ref SVC_S: HashMap<Point, HashSet<Point>> = preprocess::load("small", "service");
    static ref SVC_M: HashMap<Point, HashSet<Point>> = preprocess::load("medium", "service");
    static ref SVC_L: HashMap<Point, HashSet<Point>> = preprocess::load("large", "service");
}

pub mod preprocess {
    use super::*;

    use serde_json::{Deserializer, Serializer};
    use serde_with::serde_as;

    use std::fmt::Error;
    use std::fs::{DirEntry, File, OpenOptions};
    use std::io::prelude::*;
    use std::io::{self, BufReader, Write};
    use std::path::Path;
    use std::fs;

    #[serde_as]
    #[derive(Serialize, Deserialize, Debug)]
    struct PointData {
        #[serde_as(as = "Vec<(_, _)>")]
        map: HashMap<Point, HashSet<Point>>,
    }
    impl PointData {
        pub fn new(map: HashMap<Point, HashSet<Point>>) -> Self {
            PointData {
                map,
            }
        }
        
        pub fn to_map(self) -> HashMap<Point, HashSet<Point>> {
            self.map
        }
    }

    pub fn setup_persistence() {
        let options = vec![
            ("small", "penalty"),
            ("medium", "penalty"),
            ("large", "penalty"),
            ("small", "service"),
            ("medium", "service"),
            ("large", "service")
        ];
        for (size, cover) in options {
            create(size, cover);
        }
    }
    /// Writes out the coverage points for the given size and cover, i.e. penalty or service.
    fn create(size: &str, cover: &str) {
        let output_path = match (size, cover) {
            ("small", "penalty") => "./preprocess/penalty/small.json",
            ("medium", "penalty") => "./preprocess/penalty/medium.json",
            ("large", "penalty") => "./preprocess/penalty/large.json",
            ("small", "service") => "./preprocess/service/small.json",
            ("medium", "service") => "./preprocess/service/medium.json",
            ("large", "service") => "./preprocess/service/large.json",
            _ => panic!("Invalid size or cover")
        };

        let r: u8 = match (size, cover) {
            ("small", "penalty") => 8,
            ("medium", "penalty") => 10,
            ("large", "penalty") => 14,
            ("small", "service") => 3,
            ("medium", "service") => 3,
            ("large", "service") => 3,
            _ => panic!("Invalid size or cover")
        };

        let dim: u8 = match size {
            "small" => 30,
            "medium" => 50,
            "large" => 100,
            _ => panic!("Invalid size")
        };

        assert!(!Path::new(output_path).exists(), "Point preprocessing for {} already exists.", size);

        let mut data: Vec<PointData> = Vec::new();
        for i in 0..dim {
            for j in 0..dim {
                let p = Point::new(i.into(), j.into());
                let points_within = Point::points_within_radius(p, r, dim);
                let mut map = HashMap::new();
                map.insert(p, points_within);
                let entry = PointData::new(map);
                data.push(entry);
            }
        }

        let j = serde_json::to_string(&data).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(output_path)
            .unwrap();
        file.write_all(j.as_bytes()).unwrap();
    }

    pub fn load(size: &str, cover: &str) -> HashMap<Point, HashSet<Point>> {
        let input_path = match (size, cover) {
            ("small", "penalty") => "./preprocess/penalty/small.json",
            ("medium", "penalty") => "./preprocess/penalty/medium.json",
            ("large", "penalty") => "./preprocess/penalty/large.json",
            ("small", "service") => "./preprocess/service/small.json",
            ("medium", "service") => "./preprocess/service/medium.json",
            ("large", "service") => "./preprocess/service/large.json",
            _ => panic!("Invalid size or cover")
        };

        let file = File::open(input_path).expect("Could not open file.");
        let reader = BufReader::new(file);

        let data: PointData = serde_json::from_reader(reader).unwrap();
        data.to_map()
    }
}


/// Represents a lattice point on the grid. Has integer x-y coordinates.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

impl fmt::Debug for Point {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl fmt::Display for Point {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl Point {
	/// Creates and returns a new Point with the given x and y coordinates.
	pub fn new(x: i32, y: i32) -> Self {
		Point { x, y }
	}

	/// Returns the Euclidean distance between two points.
	fn dist(p1: &Point, p2: &Point) -> f64 {
		(((p1.x - p2.x).pow(2) + (p1.y - p2.y).pow(2)) as f64).sqrt()
	}

	/// Returns the Euclidean distance between this point and the given point.
	fn dist_to(&self, p: &Point) -> f64 {
		Point::dist(self, p)
	}

	/// Returns the file string form of this point, e.g. (3, 4) -> "3 4".
	pub fn file_string(&self) -> String {
		self.x.to_string() + " " + &self.y.to_string()
	}

	pub fn get_x(&self) -> i32 {
		self.x
	}

	pub fn get_y(&self) -> i32 {
		self.y
	}

	/// Returns a set of all the grid points within the given radius of the given
	/// point.
	pub fn points_within_radius(p: Point, r: u8, dim: u8) -> HashSet<Point> {
		// let mut result = HashSet::new();
		// let r = r as i32;
		// for i in -r..(r + 1) {
		// 	for j in -r..(r + 1) {
		// 		if Self::within(r, p.x, p.y, p.x + i, p.y + j, dim) {
		// 			result.insert(Self::new(p.x + i, p.y + j));
		// 		}
		// 	}
		// }

		// result

        let result = match (dim, r) {
            (30, 8) => PEN_S.get(&p),
            (50, 10) => PEN_M.get(&p),
            (100, 14) => PEN_L.get(&p),
            (30, 3) => SVC_S.get(&p),
            (50, 3) => SVC_M.get(&p),
            (100, 3) => SVC_L.get(&p),
            _ => panic!("Invalid size / radius combination")
        };
        result.unwrap().clone()
	}

	/// Returns whether (x2, y2) is within r units of (x1, y1) and within this
	/// Grid.
	fn within(r: i32, x1: i32, y1: i32, x2: i32, y2: i32, d: u8) -> bool {
		if x2 < 0 || x2 >= d as i32 || y2 < 0 || y2 >= d as i32 {
			return false;
		}
		(x1 - x2).pow(2) + (y1 - y2).pow(2) <= r.pow(2)
	}
}

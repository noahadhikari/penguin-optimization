mut struct Grid {
    dimension: usize,
    service_radius: u8,
    penalty_radius: u8,
    mut points: Vec<Vec<GridPoint>>,
    mut penalty: u64,
}

impl Grid {
    pub fn new(dimension: usize, service_radius: u8, penalty_radius: u8) -> Grid {
        Grid {
            dimension,
            service_radius,
            penalty_radius,
            points: new_points(dimension),
            penalty: 0,
        }
    }

    fn new_points(dimension: usize) -> Vec<Vec<GridPoint>> {
        let mut points = Vec::with_capacity(dimension);
        for i in 0..dimension {
            let mut row = Vec::with_capacity(dimension);
            for j in 0..dimension {
                row.push(GridPoint::new(i, j));
            }
            points.push(row);
        }
    }
}

struct GridPoint {
    x: u8,
    y: u8,
    tower: bool,
    city: bool,
};

impl GridPoint {
    fn new(x: u8, y: u8) -> GridPoint {
        GridPoint {
            x,
            y,
            tower: false,
            city: false,
        }
    }
}
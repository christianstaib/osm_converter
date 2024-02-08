use crate::sphere::geometry::{arc::Arc, point::Point};

#[derive(Clone)]
pub struct ConvecQuadrilateral {
    pub outline: Vec<Point>,
}

impl ConvecQuadrilateral {
    pub fn split(&self) -> Vec<ConvecQuadrilateral> {
        let arcs: Vec<Arc> = self
            .outline
            .windows(2)
            .map(|arc| Arc::new(&arc[0], &arc[1]))
            .collect();

        let o = self.outline.clone();

        let m: Vec<Point> = arcs.iter().map(|arc| arc.middle()).collect();

        let d0 = Arc::new(&m[0], &m[2]);
        let d1 = Arc::new(&m[1], &m[3]);

        let middle = d0.intersection(&d1).expect("should intersection");

        let mut subs = Vec::new();
        let p0 = ConvecQuadrilateral::new(&vec![m[3], middle, m[2], o[3], m[3]]);
        subs.push(p0);

        let p1 = ConvecQuadrilateral::new(&vec![middle, m[1], o[2], m[2], middle]);
        subs.push(p1);

        let p2 = ConvecQuadrilateral::new(&vec![m[0], o[1], m[1], middle, m[0]]);
        subs.push(p2);

        let p3 = ConvecQuadrilateral::new(&vec![o[0], m[0], middle, m[3], o[0]]);
        subs.push(p3);

        subs
    }
}

impl ConvecQuadrilateral {
    pub fn new(outline: &Vec<Point>) -> ConvecQuadrilateral {
        ConvecQuadrilateral {
            outline: outline.clone(),
        }
    }

    pub fn get_midpoint(&self) -> Point {
        for _ in 0..5 {
            let m0 = Arc::new(&self.outline[0], &self.outline[1]).random_intermediate_point();
            let m1 = Arc::new(&self.outline[1], &self.outline[2]).random_intermediate_point();
            let m2 = Arc::new(&self.outline[2], &self.outline[3]).random_intermediate_point();
            let m3 = Arc::new(&self.outline[3], &self.outline[4]).random_intermediate_point();
            let d0 = Arc::new(&m0, &m2);
            let d1 = Arc::new(&m1, &m3);
            if let Some(intersection) = d0.intersection(&d1) {
                return intersection;
            }
        }
        panic!("no midpoint found :(");
    }

    pub fn intersections(&self, line: &Arc) -> Vec<Point> {
        self.outline
            .windows(2)
            .filter_map(|outline| {
                let outline = Arc::new(&outline[0], &outline[1]);
                line.intersection(&outline)
            })
            .collect()
    }
}

pub struct Tiling {}

impl Tiling {
    pub fn base_tiling() -> Vec<ConvecQuadrilateral> {
        let np = Point::from_coordinate(90.0, 0.0);
        let sp = Point::from_coordinate(-90.0, 0.0);
        let mid_ring: Vec<f64> = vec![180.0, -90.0, 0.0, 90.0, 180.0];
        let mid_ring: Vec<Point> = mid_ring
            .iter()
            .map(|&lon| Point::from_coordinate(0.0, lon))
            .collect();
        let upper_ring: Vec<Point> = mid_ring
            .iter()
            .map(|mid| Arc::new(mid, &np).middle())
            .collect();
        let lower_ring: Vec<Point> = mid_ring
            .iter()
            .map(|mid| Arc::new(mid, &sp).middle())
            .collect();
        let mid_ring: Vec<f64> = vec![-135.0, -45.0, 45.0, 135.0, -135.0];
        let mid_ring: Vec<Point> = mid_ring
            .iter()
            .map(|&lon| Point::from_coordinate(0.0, lon))
            .collect();

        let mut base_pixels = Vec::new();

        for i in 0..4 {
            let polygon = ConvecQuadrilateral::new(&vec![
                upper_ring[i],
                mid_ring[i],
                upper_ring[i + 1],
                np,
                upper_ring[i],
            ]);
            base_pixels.push(polygon);

            let polygon = ConvecQuadrilateral::new(&vec![
                lower_ring[i],
                sp,
                lower_ring[i + 1],
                mid_ring[i],
                lower_ring[i],
            ]);
            base_pixels.push(polygon);

            let polygon = ConvecQuadrilateral::new(&vec![
                mid_ring[i],
                lower_ring[i + 1],
                mid_ring[i + 1],
                upper_ring[i + 1],
                mid_ring[i],
            ]);
            base_pixels.push(polygon);
        }
        base_pixels
    }
}

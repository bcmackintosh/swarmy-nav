use std::collections::{HashMap, hash_map::Entry};
use std::ops::{Index, IndexMut};
use std::f64::consts::{PI, FRAC_PI_2};

pub type Radius = f64;
pub type Angle = f64;

enum Quadrant {
    TopRight,
    TopLeft,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Debug)]
pub struct PolarCoordinates {
    pub origin: usize,
    pub radials: HashMap<usize, Radial>,
}

impl Index<&'_ usize> for PolarCoordinates {
    type Output = Radial;
    fn index(&self, k: &usize) -> &Radial {
        &self.radials[k]
    }
}

impl IndexMut<&'_ usize> for PolarCoordinates {
    fn index_mut(&mut self, k: &usize) -> &mut Radial {
        match self.radials.get_mut(k) {
            Some(r) => r,
            None => panic!("No radial found at that value.")
        }
    }
}

impl PolarCoordinates {
    pub fn new(origin: usize) -> PolarCoordinates {
        PolarCoordinates {
            origin: origin,
            radials: HashMap::<usize, Radial>::new(),
        }
    }

    pub fn from_distances(origin: usize, calibration_idx: usize, skip_ids: Vec<usize>, calibration: &Vec<(usize, f64)>, distance_vec: &Vec<Vec<f64>>) -> PolarCoordinates {
        let mut output = PolarCoordinates {
            origin: origin,
            radials: HashMap::<usize, Radial>::new(),
        };

        // let skip_ids: Vec<usize> = calibration.iter().map(|x| x.0).collect();

        output.add_radial(Radial {
            id: calibration[calibration_idx].0,
            radius: calibration[calibration_idx].1,
            angle: 0.0 as Angle
        });
        
        for i in 1..distance_vec.len() {
            if skip_ids.contains(&i) {
                continue;
            }
            
            let a = distance_vec[0][calibration[calibration_idx].0];
            let b = distance_vec[0][i];
            let c = distance_vec[calibration[calibration_idx].0][i];
            output.add_b_edge(i, a, b, c);
        }

        return output;
    }

    pub fn add_radial(&mut self, r: Radial) {
        self.radials.insert(r.id, r);
    }

    pub fn add_b_edge(&mut self, id: usize, a: f64, b: f64, c: f64) {
         self.radials.insert(id, Radial::from_distances(id, a, b, c));
    }

    pub fn get(&self, key: &usize) -> Option<&Radial> {
        return self.radials.get(key);
    }

    pub fn get_mut(&mut self, key: &usize) -> Option<&mut Radial> {
        return self.radials.get_mut(key);
    }

    pub fn reconcile_radial(&mut self, offset_angle: Angle, id: usize, radial: &Radial) {
        let test_radial = self.get_mut(&id).unwrap();

        let pos_angle = radial.angle + offset_angle;
        let neg_angle = radial.angle - offset_angle;

        if (test_radial.angle - pos_angle).abs() < 0.0000001 {
            return;
        }

        test_radial.angle = neg_angle;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Radial {
    pub id: usize,
    pub radius: Radius,
    pub angle: Angle
}

impl Radial {
    pub fn empty(id: usize) -> Radial {
        Radial {
            id: id,
            radius: 0.0,
            angle: 0.0
        }
    }

    pub fn from_distances(id: usize, a: f64, b: f64, c:f64) -> Radial {
        Radial {
            id: id,
            radius: b,
            angle: get_unknown_triangle_angle(a, b, c)
        }
    }
    pub fn to_degrees(&self) -> Radial {
        return Radial
        {
            id: self.id,
            radius: self.radius,
            angle: self.angle.to_degrees()
        };
    }

    pub fn get_cartesian(&self) -> (f64, f64) {
        let x = (FRAC_PI_2 - self.angle).sin() * self.radius;
        let y = self.angle.sin() * self.radius;

        return (x, y);
    }

    pub fn subtract(&self, other: &Radial) -> Radial {
        let (s_x, s_y) = self.get_cartesian();
        let (o_x, o_y) = other.get_cartesian();

        let x = o_x - s_x;
        let y = o_y - s_y;

        let radius = (x.powi(2) + y.powi(2)).sqrt();
        let angle = (y / radius).asin();

        return Radial {
            id: self.id,
            radius: radius,
            angle: match get_quadrant_from_cartesian(x, y) {
                Quadrant::TopRight => angle,
                Quadrant::TopLeft => {
                    PI - angle
                },
                Quadrant::BottomLeft => {
                    angle.abs() + PI
                },
                Quadrant::BottomRight => angle,
            }
        }
    }
}

pub fn get_unknown_triangle_angle(a: f64, b: f64, c: f64) -> Angle {
    if a == 0.0 || b == 0.0 || c == 0.0 {
        return 0.0;
    }

    return ((a.powi(2) + b.powi(2) - c.powi(2)) / (2 as f64 * a * b)).acos();
}

pub fn get_unknown_triangle_side(a: f64, b: f64, theta: Angle) -> f64 {
    if theta == 0.0 {
        return (a - b).abs();
    }

    if a == 0.0 || b == 0.0 {
        return 0.0;
    }

    // Law of Cosines:
    // a^2 + b^2 - c^2 / 2 * a * b = cos(theta)
    // sqrt(a^2 + b^2 - cos(theta) * 2 * a * b) = c

    return (a.powi(2) + b.powi(2) - 2 as f64 * a * b * theta.cos()).sqrt();
}

pub fn add_radials(radials: &Vec<Radial>) -> Radial {
    let mut x = 0.0;
    let mut y = 0.0;
    let id = radials[0].id;
    // Law of Sines:
    // a / sin(a) = c / sin(c)
    // sin(pi/2) == 1

    for radial in radials {
        x += (FRAC_PI_2 - radial.angle).sin() * radial.radius;
        y += radial.angle.sin() * radial.radius;
    }


    let radius = (x.powi(2) + y.powi(2)).sqrt();
    let angle = (y / radius).asin();

    return Radial {
        id: id,
        radius: radius,
        angle: match get_quadrant_from_cartesian(x, y) {
            Quadrant::TopRight => angle,
            Quadrant::TopLeft => {
                PI - angle
            },
            Quadrant::BottomLeft => {
                angle.abs() + PI
            },
            Quadrant::BottomRight => angle,
        }
    };
}

fn get_quadrant_from_cartesian(x: f64, y: f64) -> Quadrant {
    if x >= 0.0 && y >= 0.0 {
        return Quadrant::TopRight;
    }

    if x < 0.0 && y < 0.0 {
        return Quadrant::BottomLeft;
    }

    if x < 0.0 {
        return Quadrant::TopLeft;
    }

    if y < 0.0 {
        return Quadrant::BottomRight;
    }

    return Quadrant::TopRight;
}
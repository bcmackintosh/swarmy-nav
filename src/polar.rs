use std::collections::{HashMap, hash_map::Entry};
use std::ops::{Index, IndexMut};
use std::f64::consts::{PI, FRAC_PI_2};
use crate::identity::Identity;

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
    pub origin: Identity,
    pub radials: HashMap<Identity, Radial>,
}

impl Index<&'_ Identity> for PolarCoordinates {
    type Output = Radial;
    fn index(&self, k: &Identity) -> &Radial {
        &self.radials[k]
    }
}

impl IndexMut<&'_ Identity> for PolarCoordinates {
    fn index_mut(&mut self, k: &Identity) -> &mut Radial {
        match self.radials.get_mut(k) {
            Some(r) => r,
            None => panic!("No radial found at that value.")
        }
    }
}

impl PolarCoordinates {
    pub fn new(origin: Identity) -> PolarCoordinates {
        PolarCoordinates {
            origin: origin,
            radials: HashMap::<Identity, Radial>::new(),
        }
    }

    pub fn from_distances(origin: Identity, calibration_id: Identity, skip_ids: Vec<Identity>, calibration: &Vec<(Identity, f64)>, distance_vec: &Vec<Vec<f64>>) -> PolarCoordinates {
        let mut output = PolarCoordinates {
            origin: origin,
            radials: HashMap::<Identity, Radial>::new(),
        };

        let calibration_idx = calibration.iter().position(|x| x.0 == calibration_id).unwrap();

        // let skip_ids: Vec<usize> = calibration.iter().map(|x| x.0).collect();

        output.add_radial(Radial {
            id: calibration[calibration_idx].clone().0,
            radius: calibration[calibration_idx].1,
            angle: 0.0 as Angle
        });

        let diff = distance_vec.len() - calibration.len();

        for i in diff..distance_vec.len() {
            let idx = i - diff;
            if skip_ids.contains(&calibration[idx].0) {
                continue;
            }
            
            let a = distance_vec[0][calibration_idx];
            let b = distance_vec[0][i];
            let c = distance_vec[calibration_idx][i];
            output.add_b_edge(calibration[idx].0.clone(), a, b, c);
        }

        return output;
    }

    pub fn add_radial(&mut self, r: Radial) {
        self.radials.insert(r.clone().id, r.clone());
    }

    pub fn add_b_edge(&mut self, id: Identity, a: f64, b: f64, c: f64) {
         self.radials.insert(id.clone(), Radial::from_distances(id.clone(), a, b, c));
    }

    pub fn get(&self, key: &Identity) -> Option<&Radial> {
        return self.radials.get(key);
    }

    pub fn get_mut(&mut self, key: &Identity) -> Option<&mut Radial> {
        return self.radials.get_mut(key);
    }

    // Takes the offset angle and compares the incoming angle based from the offset +/-.  The angle is kept if the offset + the incoming angle equal the given angle.  If not the kept angle is equal to the offset - the incoming angle.  This allows for negative angles, since distances will always calculate angles <180 degrees, but have no way to determine clockwise vs counterclockwise.

    // The positive and negative direction on the angle is relative to an "offset" angle direction and not an absolute clockwise/counterclockwise.
    pub fn reconcile_radial(&mut self, offset_angle: Angle, id: Identity, radial: &Radial) {
        let test_radial = self.get_mut(&id).unwrap();

        let pos_angle = offset_angle + radial.angle;
        let neg_angle = offset_angle - radial.angle;

        if (test_radial.angle - pos_angle).abs() < 0.0000001 {
            return;
        }

        test_radial.angle = neg_angle;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Radial {
    pub id: Identity,
    pub radius: Radius,
    pub angle: Angle
}

impl Radial {
    pub fn empty(id: Identity) -> Radial {
        Radial {
            id: id,
            radius: 0.0,
            angle: 0.0
        }
    }

    pub fn from_distances(id: Identity, a: f64, b: f64, c:f64) -> Radial {
        Radial {
            id: id,
            radius: b,
            angle: get_unknown_triangle_angle(a, b, c)
        }
    }
    pub fn to_degrees(&self) -> Radial {
        return Radial
        {
            id: self.id.clone(),
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
            id: self.id.clone(),
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
    let id = radials[0].id.clone();
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
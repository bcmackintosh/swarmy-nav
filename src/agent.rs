use crate::{polar::{Angle, Radial, get_unknown_triangle_side, get_unknown_triangle_angle, add_radials}, identity::Identity};

use std::{time::SystemTime, alloc::System};
use std::f64::consts::PI;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Agent {
    pub origin: usize,
    pub position: Radial,
    pub current_coord: Option<Radial>,
    pub path: VecDeque<Radial>,
    // In distance per ms.  This is just a fake agent that we will eventually replace with real velocity.  This would need something like current velocity and a max velocity, with the ability to accelerate.  Since we don't actually need a smooth velocity (yet) this just approximates the data points the caller gets over time.
    pub velocity: f64,
    pub last_update: SystemTime
}

impl Agent {
    pub fn new(origin: usize, position: Radial, velocity: f64) -> Agent {
        Agent {
            origin: origin,
            position: position.clone(),
            current_coord: None,
            path: VecDeque::new(),
            velocity: velocity,
            last_update: SystemTime::now()
        }
    }

    fn path_next(&mut self) {
        let next_coord = self.path.pop_front();
        
        if next_coord.is_none() {
            self.current_coord = None;
            return;
        }

        self.current_coord = Some(self.position.subtract(&next_coord.as_ref().unwrap()));
        return;
    }

    fn update_position(&mut self) {
        if self.current_coord.is_none() && self.path.len() == 0 {
            return;
        }

        let now = SystemTime::now();
        let difference = now.duration_since(self.last_update).unwrap().as_micros() as f64 / 1_000.0;
        self.last_update = now;

        let mut distance_to_travel = difference * self.velocity;
        let mut radials: Vec<Radial> = vec![self.position.clone()];

        while distance_to_travel > 0.0 && self.current_coord.is_some() {
            distance_to_travel -= self.current_coord.as_ref().unwrap().radius;

            if distance_to_travel > 0.0 {
                radials.push(self.current_coord.as_ref().unwrap().clone());
                self.position = add_radials(&radials);
                self.path_next();
                radials = vec![self.position.clone()];

                continue;
            }

            let coord = self.current_coord.as_ref().unwrap();

            radials.push(Radial { 
                id: coord.id, 
                radius: coord.radius + distance_to_travel, 
                angle: coord.angle });
            self.position = add_radials(&radials);
            radials = vec![self.position.clone()];

            self.current_coord.as_mut().unwrap().radius = distance_to_travel * -1.0;
        }
        
    }

    pub fn send_position(&mut self, position: &Radial) {
        self.path.push_back(position.clone());
        
        if self.current_coord.is_none() {
            self.path_next();
        }
    }
    
    pub fn get_position(&mut self) -> Radial {
        self.update_position();

        return self.position.clone();
    }
}
use ggez::nalgebra as na;
use rand::thread_rng;
use std::f32::consts::PI;
use rand::{Rng, prelude::ThreadRng};

pub struct Home {
    pub position: na::Point2<f32>,
    pub radius: f32,
    rng: ThreadRng,
}

impl Home {
    pub fn new (position: na::Point2<f32>, radius: f32) -> Self {
        Home {
            position: position,
            radius: radius,
            rng: thread_rng(),
        }
    } 

    pub fn generate_starting_position (&mut self) -> (na::Point2<f32>, f32) {
        let random_value : f32 = self.rng.gen();  
        let angle = random_value * 2.0 * PI;
        let current_x = angle.cos() * (self.radius + 5.0) + self.position.x;
        let current_y = angle.sin() * (self.radius + 5.0) + self.position.y;

        (na::Point2::new(current_x, current_y), angle)
    }

    pub fn touching_home (&self, position: na::Point2<f32>) -> bool {
        let touching = (position - self.position).norm() <= self.radius;
        return touching;
    }
}
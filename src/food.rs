use ggez::nalgebra as na;
use crate::TOTAL_FOOD;
use rand::{Rng, thread_rng, prelude::ThreadRng};

pub struct Food {
    pub position: [Option<na::Point2<f32>>; TOTAL_FOOD],
    pub state: [u32; TOTAL_FOOD],
    pub food_generated: u32,
    rng: ThreadRng,
}

impl Food {
    pub fn new () -> Self {
        Food {
            position: [None; TOTAL_FOOD],
            state: [0; TOTAL_FOOD],
            food_generated: 0,
            rng: thread_rng(),
        }
    }

    pub fn add_food (&mut self, bottom_left: na::Point2<f32>, top_right: na::Point2<f32>, food: u32) {
        for i in self.food_generated..(self.food_generated+food) {
            let current_x = self.rng.gen_range(bottom_left.x..top_right.x);
            let current_y = self.rng.gen_range(bottom_left.y..top_right.y);
            self.position[i as usize] = Some(na::Point2::new(current_x, current_y));
            self.state[i as usize] = 0;
        }

        self.food_generated += food;
    }
}
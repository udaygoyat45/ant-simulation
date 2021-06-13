use ggez::nalgebra as na;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::f32;
use std::f32::consts::PI;
use rayon::prelude::*;

use crate::utils;
use crate::TOTAL_ANTS;

#[derive(Copy, Clone)]
pub struct Ant {
    max_speed : [f32; TOTAL_ANTS],
    steer_strength: f32,
    wander_strength: f32,
    rust_factor: f32,

    pub angle: [f32; TOTAL_ANTS],
    pub position: [na::Point2<f32>; TOTAL_ANTS],
    pub velocity: [na::Vector2<f32>; TOTAL_ANTS],
    pub  desired_direction: [na::Vector2<f32>; TOTAL_ANTS],

    home_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS],
    food_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS],
    
    pub state: [u32; TOTAL_ANTS],

    pub target_food_position: [Option<na::Point2<f32>>; TOTAL_ANTS],
    pub target_food_index: [Option<u32>; TOTAL_ANTS],
    window_size: (f32, f32),
}

impl Ant {
    pub fn initialize(&mut self, init_data: &[(na::Point2<f32>, f32)]) {
        for i in 0..TOTAL_ANTS {
            self.angle[i] = init_data[i].1;
            self.position[i] = init_data[i].0;
            self.velocity[i] = na::Vector2::new(init_data[i].0.x.cos(), init_data[i].0.y.sin()) * self.max_speed[i];
            self.desired_direction[i] = na::Vector2::new(init_data[i].1.cos(), init_data[i].1.sin());
        }
    }

    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        let new_angle =  [0.0; TOTAL_ANTS];
        let new_position =  [na::Point2::new(screen_w/2.0, screen_h/2.0); TOTAL_ANTS];
        let new_velocity =  [na::Vector2::new(1.0 as f32, 0.0 as f32); TOTAL_ANTS];
        let new_desired_direction =  [na::Vector2::new(1.0 as f32, 0.0 as f32); TOTAL_ANTS];
        let new_home_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS] =  [None; TOTAL_ANTS];
        let new_food_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS] =  [None; TOTAL_ANTS];
        let new_state: [u32; TOTAL_ANTS] = [0; TOTAL_ANTS];
        let new_target_food_position: [Option<na::Point2<f32>>; TOTAL_ANTS] =  [None; TOTAL_ANTS];
        let new_target_food_index: [Option<u32>; TOTAL_ANTS] =  [None; TOTAL_ANTS];

        Ant {
            max_speed: [50.0; TOTAL_ANTS],
            steer_strength: 100.0,
            wander_strength: 0.1,
            rust_factor: 50.0,
            angle: new_angle,
            position: new_position,
            velocity: new_velocity,
            desired_direction: new_desired_direction,
            home_pheromones_direction: new_home_pheromones_direction,
            food_pheromones_direction: new_food_pheromones_direction,
            state: new_state,
            target_food_position: new_target_food_position,
            target_food_index: new_target_food_index,
            window_size: (screen_w, screen_h),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let wander_strength = self.wander_strength;
        let window_size = self.window_size;
        let steer_strength = self.steer_strength;
        let rust_factor = self.rust_factor;

        self.max_speed.par_iter_mut()
            .zip(self.angle.par_iter_mut())
            .zip(self.position.par_iter_mut())
            .zip(self.velocity.par_iter_mut())
            .zip(self.desired_direction.par_iter_mut())
            .zip(self.food_pheromones_direction.par_iter_mut())
            .zip(self.home_pheromones_direction.par_iter_mut())
            .zip(self.state.par_iter_mut())
            .zip(self.target_food_position.par_iter_mut())
            .for_each(|((((((((max_speed, 
                    angle),
                    position),
                    velocity),
                    desired_direction),
                    food_pheromones_direction),
                    home_pheromones_direction),
                    state),
                    target_food_position)| {

            let mut rng = StdRng::from_entropy();
            let random_unit_vector = na::Vector2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
            if *state == 0 {
                match food_pheromones_direction {
                    Some(j) => *desired_direction = *j,
                    None => {
                        *desired_direction = *desired_direction + random_unit_vector * wander_strength;
                        *desired_direction = desired_direction.normalize();
                    }
                }
            } else if *state == 1 {
                match target_food_position {
                    Some(j) => {
                        *desired_direction = na::Vector2::from(*j - *position).normalize();
                    },
                    None => {
                        *desired_direction = *desired_direction + random_unit_vector * wander_strength;
                        *desired_direction = desired_direction.normalize();
                    },
                }
            } else {
                let to_home = na::Point2::new(window_size.0, window_size.1)/2.0 - *position;
                let dist_sq = na::distance_squared(&(na::Point2::new(window_size.0, window_size.1)/2.0), &position);
                if dist_sq < f32::powi(100.0, 2) {
                    *desired_direction = na::Vector::from(to_home);
                } else {
                    match home_pheromones_direction {
                        None => {
                            *desired_direction = *desired_direction + random_unit_vector * wander_strength;
                            *desired_direction = desired_direction.normalize();
                        },
                        Some(j) => {
                            *desired_direction = *j;
                        }
                    }
                }
            }

            let desired_velocity = *desired_direction * *max_speed;
            let desired_steering_force = (desired_velocity - *velocity) * steer_strength;
            let acceleration = utils::clamp_magnitude(&desired_steering_force, steer_strength);

            let new_velocity = utils::clamp_magnitude(&(*velocity + acceleration * dt),*max_speed);

            if !new_velocity.x.is_nan() && !new_velocity.y.is_nan() {
                *velocity = new_velocity;
            }

            let position_increment = *velocity * dt; // * rust_factor;
            *position += position_increment;

            // if position.x > window_size.0 {position.x = 0.0;}
            // if position.x < 0.0 {position.x = window_size.0;}
            // if position.y > window_size.1 {position.y = 0.0;}
            // if position.y < 0.0 {position.y = window_size.1;}

            if position.x > window_size.0 {*velocity = -*velocity; *desired_direction = -*desired_direction;}
            if position.x < 0.0 {*velocity = -*velocity; *desired_direction = -*desired_direction;}
            if position.y > window_size.1 {*velocity = -*velocity; *desired_direction = -*desired_direction;}
            if position.y < 0.0 {*velocity = -*velocity; *desired_direction = -*desired_direction;}

            *angle = (velocity.y/velocity.x).atan();
            if velocity[0] < 0.0 {*angle -= PI}
        });
    }

    pub fn set_food_target(&mut self, index: usize, food_position: na::Point2<f32>, food_index: u32) {
        if self.state[index] == 0 {
            self.state[index] = 1;
            self.target_food_index[index] = Some(food_index);
            self.target_food_position[index] = Some(food_position);
            self.desired_direction[index] = na::Vector2::from(food_position - self.position[index]).normalize();
        }
    }

    pub fn food_acquired(&mut self, index: usize) -> bool {
        match self.target_food_position[index] {
            None => false,
            Some(food_position) => {
                self.state[index] == 1 && (food_position - self.position[index]).norm() < 5.0
            }
        }
    }

    pub fn set_pheromones_direction(&mut self, index: usize, home_angle: Option<f32>, food_angle: Option<f32>) {
        match home_angle {
            None => {self.home_pheromones_direction[index] = None; },
            Some(j) => {self.home_pheromones_direction[index] = Some(na::Vector2::new(j.cos(), j.sin()));}
        }

        match food_angle {
            None => {self.food_pheromones_direction[index] = None; },
            Some(j) => {self.food_pheromones_direction[index] = Some(na::Vector2::new(j.cos(), j.sin()));}
        }
    }

    pub fn set_antiparallel (&mut self, index: usize) {
        self.velocity[index] = -self.velocity[index];
    }

}

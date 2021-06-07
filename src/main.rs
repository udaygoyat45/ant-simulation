use ggez::{Context, ContextBuilder, GameResult};
// use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;
use ggez::graphics::{self};
use rand::Rng;
use std::f32;
use std::f32::consts::PI;
use rayon::prelude::*;

const TOTAL_ANTS: usize = 500;
const ANT_RADIUS: f32 = 2.0;

fn clamp_magnitude(x: &na::Vector2<f32>, c: f32) -> na::Vector2<f32> {
    x * (c / x.norm())
}

#[derive(Copy, Clone)]
struct Ant {
    max_speed : f32,
    steer_strength: f32,
    wander_strength: f32,
    rust_factor: f32,

    angle: [f32; TOTAL_ANTS],
    position: [na::Point2<f32>; TOTAL_ANTS],
    velocity: [na::Vector2<f32>; TOTAL_ANTS],
    desired_direction: [na::Vector2<f32>; TOTAL_ANTS],

    home_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS],
    food_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS],
    
    state: [u8; TOTAL_ANTS],

    target_food_position: [Option<na::Point2<f32>>; TOTAL_ANTS],
    target_food_index: [Option<u32>; TOTAL_ANTS],
    window_size: (f32, f32),
}

impl Ant {
    pub fn initialize(&mut self, angles: &[f32], init_positions: &[na::Point2<f32>]) {
        for i in 1..TOTAL_ANTS {
            self.angle[i] = angles[i];
            self.position[i] = init_positions[i];
            self.velocity[i] = na::Vector2::new(init_positions[i].x.cos(), init_positions[i].y.sin()) * self.max_speed;
            self.desired_direction[i] = na::Vector2::new(angles[i].cos(), angles[i].sin());
        }
    }

    pub fn empty(screen_w: f32, screen_h: f32) -> Self {
        let new_angle =  [0.0; TOTAL_ANTS];
        let new_position =  [na::Point2::new(screen_w/2.0, screen_h/2.0); TOTAL_ANTS];
        let new_velocity =  [na::Vector2::new(1.0 as f32, 0.0 as f32); TOTAL_ANTS];
        let new_desired_direction =  [na::Vector2::new(1.0 as f32, 0.0 as f32); TOTAL_ANTS];
        let new_home_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS] =  [None; TOTAL_ANTS];
        let new_food_pheromones_direction: [Option<na::Vector2<f32>>; TOTAL_ANTS] =  [None; TOTAL_ANTS];
        let new_state: [u8; TOTAL_ANTS] = [0; TOTAL_ANTS];
        let new_target_food_position: [Option<na::Point2<f32>>; TOTAL_ANTS] =  [None; TOTAL_ANTS];
        let new_target_food_index: [Option<u32>; TOTAL_ANTS] =  [None; TOTAL_ANTS];

        Ant {
            max_speed: 1.3,
            steer_strength: 5.0,
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
        let mut rng = rand::thread_rng();

        for i in 0..TOTAL_ANTS {
            let random_unit_vector = na::Vector2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
            if self.state[i] == 0 {
                match self.food_pheromones_direction[i] {
                    Some(j) => self.desired_direction[i] = j,
                    None => {
                        self.desired_direction[i] = self.desired_direction[i] + random_unit_vector * self.wander_strength;
                        self.desired_direction[i] = self.desired_direction[i].normalize();
                    }
                }
            } else if self.state[i] == 1 {
                match self.target_food_position[i] {
                    Some(j) => {
                        let difference = j - self.position[i];
                        self.desired_direction[i] = na::Vector2::new(difference.x, difference.y).normalize();
                    },
                    None => (),
                }
            } else {
                match self.home_pheromones_direction[i] {
                    None => {
                        self.desired_direction[i] = self.desired_direction[i] + random_unit_vector * self.wander_strength;
                        self.desired_direction[i] = self.desired_direction[i].normalize();
                    },
                    Some(j) => {
                        let to_home = na::Point2::new(self.window_size.0, self.window_size.1)/2.0 - self.position[i];
                        let dist_sq = na::distance_squared(&(na::Point2::new(self.window_size.0, self.window_size.1)/2.0), &self.position[i]);
                        if dist_sq < f32::powi(88.0, 2) {
                            self.desired_direction[i] = na::Vector2::new(to_home.x, to_home.y);
                        } else {
                            self.desired_direction[i] = j;
                        }
                    }
                }
            }

            let desired_velocity = self.desired_direction[i] * self.max_speed;
            let desired_steering_force = (desired_velocity - self.velocity[i]) * self.steer_strength;
            let acceleration = clamp_magnitude(&desired_steering_force, self.steer_strength);
            let difference_angle = desired_velocity.angle(&self.velocity[i]);
            
            if difference_angle < 50.0 || difference_angle > 310.0 {
                if self.state[i] == 0 && !self.food_pheromones_direction[i].is_none() {
                    self.max_speed = 0.5;
                }
                if self.state[i] == 1 {
                    self.max_speed = 0.5;
                }
                if self.state[i] == 2 && !self.home_pheromones_direction[i].is_none() {
                    self.max_speed = 0.5;
                }
            }

            let new_velocity = clamp_magnitude(&(self.velocity[i] + acceleration * dt), self.max_speed);
            self.velocity[i] = new_velocity;

            let position_increment = self.velocity[i] * dt * self.rust_factor;
            self.position[i] += position_increment;

            if self.position[i].x > self.window_size.0 {self.position[i].x = 0.0;}
            if self.position[i].x < 0.0 {self.position[i].x = self.window_size.0;}
            if self.position[i].y > self.window_size.1 {self.position[i].y = 0.0;}
            if self.position[i].y < 0.0 {self.position[i].y = self.window_size.1;}

            self.angle[i] = (self.velocity[i].y/self.velocity[i].x).atan();
            if self.velocity[i][0] < 0.0 {self.angle[i] -= PI}
        }
    }

}
struct MainState {
    ants: Ant,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let (mut screen_w, mut screen_h) = graphics::drawable_size(ctx);
        screen_w /= 2.0;
        screen_h /= 2.0;

        let current_ants = Ant::empty(screen_w, screen_h);
        MainState {
            ants: current_ants,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        self.ants.update(dt);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        let mut draw_param = graphics::DrawParam::default();

        for ant_index in 0..TOTAL_ANTS {
            let current_ant_mesh = graphics::Mesh::new_ellipse(ctx, 
                graphics::DrawMode::fill(),
                self.ants.position[ant_index], ANT_RADIUS,
                ANT_RADIUS, graphics::FillOptions::DEFAULT_TOLERANCE,
                graphics::WHITE).unwrap();
            draw_param.dest = self.ants.position[ant_index].into();
            graphics::draw(ctx, &current_ant_mesh, draw_param).unwrap();
        }

        // for ant_position in self.ants.position.iter() {
        // }

        graphics::present(ctx).unwrap();
        Ok(())
    }
}

fn main() {
    let cb = ContextBuilder::new("Ant Simulation", "Some One");
    let (mut ctx, mut event_loop) = cb.build().unwrap();
    graphics::set_window_title(&ctx, "Ant Simulation");
    
    // let custom_window_config = conf::WindowMode {
    //     width: WIDTH+500.0,
    //     height: HEIGHT,
    //     ..Default::default()
    // };

    // graphics::set_mode(&mut ctx, custom_window_config).unwrap();

    let (screen_w, screen_h) = graphics::drawable_size(&mut ctx);
    println!("declared width: {}, declared height: {}", screen_w, screen_h);

    // graphics::set_drawable_size(&mut ctx, WIDTH, HEIGHT).unwrap();
    // graphics::set_screen_coordinates(&mut ctx, graphics::Rect::new(0.0, 0.0, WIDTH, HEIGHT)).unwrap();
    graphics::set_window_title(&mut ctx, "Ant Simulation");

    let mut state = MainState::new(&mut ctx);
    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}
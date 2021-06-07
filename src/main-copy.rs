use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;
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

    angle: f32,
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
    desired_direction: na::Vector2<f32>,

    home_pheromones_direction: Option<na::Vector2<f32>>,
    food_pheromones_direction: Option<na::Vector2<f32>>,
    
    state: u8,

    target_food_position: Option<na::Point2<f32>>,
    target_food_index: Option<na::Point2<f32>>,
    window_size: (f32, f32),
}

impl Ant {
    pub fn initialize(&mut self, angle: f32, init_position: na::Point2<f32>) {
        self.position = init_position;
        self.angle = angle;
        self.velocity = na::Vector2::new(init_position.x.cos(), init_position.y.sin()) * self.max_speed;
        self.desired_direction = na::Vector2::new(angle.cos(), angle.sin());
    }

    pub fn empty(screen_w: f32, screen_h: f32) -> Self {
        Ant {
            max_speed: 1.3,
            steer_strength: 5.0,
            wander_strength: 0.1,
            rust_factor: 50.0,
            angle: 0.0,
            position: na::Point2::new(screen_w/2.0, screen_h/4.0),
            velocity: na::Vector2::new(1.0, 0.0),
            desired_direction: na::Vector2::new(1.0, 0.0),
            home_pheromones_direction: None,
            food_pheromones_direction: None,
            state: 0,
            target_food_position: None,
            target_food_index: None,
            window_size: (screen_w, screen_h),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut rng = rand::thread_rng();
        let random_unit_vector = na::Vector2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));

        if self.state == 0 {
            match self.food_pheromones_direction {
                Some(i) => self.desired_direction = i,
                None => {
                    self.desired_direction = self.desired_direction + random_unit_vector * self.wander_strength;
                    self.desired_direction = self.desired_direction.normalize();
                }
            }
        } else if self.state == 1 {
            match self.target_food_position {
                Some(i) => {
                    let difference = i - self.position;
                    self.desired_direction = na::Vector2::new(difference.x, difference.y).normalize();
                },
                None => (),
            }
        } else {
            match self.home_pheromones_direction {
                None => {
                    self.desired_direction = self.desired_direction + random_unit_vector * self.wander_strength;
                    self.desired_direction = self.desired_direction.normalize();
                },
                Some(i) => {
                    let to_home = na::Point2::new(self.window_size.0, self.window_size.1)/2.0 - self.position;
                    let dist_sq = na::distance_squared(&(na::Point2::new(self.window_size.0, self.window_size.1)/2.0), &self.position);
                    if dist_sq < f32::powi(88.0, 2) {
                        self.desired_direction = na::Vector2::new(to_home.x, to_home.y);
                    } else {
                        self.desired_direction = i;
                    }
                }
            }
        }

        let desired_velocity = self.desired_direction * self.max_speed;
        let desired_steering_force = (desired_velocity - self.velocity) * self.steer_strength;
        let acceleration = clamp_magnitude(&desired_steering_force, self.steer_strength);
        let difference_angle = desired_velocity.angle(&self.velocity);
        
        if difference_angle < 50.0 || difference_angle > 310.0 {
            if self.state == 0 && !self.food_pheromones_direction.is_none() {
                self.max_speed = 0.5;
            }
            if self.state == 1 {
                self.max_speed = 0.5;
            }
            if self.state == 2 && !self.home_pheromones_direction.is_none() {
                self.max_speed = 0.5;
            }
        }

        let new_velocity = clamp_magnitude(&(self.velocity + acceleration * dt), self.max_speed);
        self.velocity = new_velocity;

        let position_increment = self.velocity * dt * self.rust_factor;
        self.position += position_increment;

        if self.position.x > self.window_size.0 {self.position.x = 0.0;}
        if self.position.x < 0.0 {self.position.x = self.window_size.0;}
        if self.position.y > self.window_size.1 {self.position.y = 0.0;}
        if self.position.y < 0.0 {self.position.y = self.window_size.1;}

        self.angle = (self.velocity.y/self.velocity.x).atan();
        if self.velocity[0] < 0.0 {self.angle -= PI}
    }
}
struct MainState {
    ants: [Ant; TOTAL_ANTS]
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let (mut screen_w, mut screen_h) = graphics::drawable_size(ctx);
        screen_w /= 2.0;
        screen_h /= 2.0;

        let mut current_ants = [Ant::empty(screen_w, screen_h); TOTAL_ANTS];
        for ant in &mut current_ants{
            ant.initialize(0.0, na::Point2::new(screen_w/2.0, screen_h/2.0)); 
        }
        MainState {
            ants: current_ants,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        self.ants.par_iter_mut().for_each(|ant| {ant.update(dt);});
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        let mut draw_param = graphics::DrawParam::default();

        for ant in self.ants.iter() {
            let current_ant_mesh = graphics::Mesh::new_ellipse(ctx, 
                graphics::DrawMode::fill(),
                ant.position, ANT_RADIUS,
                ANT_RADIUS, graphics::FillOptions::DEFAULT_TOLERANCE,
                graphics::WHITE).unwrap();

            draw_param.dest = ant.position.into();
            graphics::draw(ctx, &current_ant_mesh, draw_param).unwrap();
        }

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
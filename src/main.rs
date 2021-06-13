use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;
use ggez::graphics;
use ggez::conf;
use std::cmp::{max, min};
use std::{f32, usize};
use arrayvec::ArrayVec;
// use rayon::prelude::*;
use std::f32::consts::PI;

mod ant;
mod home;
mod utils;
mod food;

const TOTAL_ANTS: usize = 1000;
const TOTAL_FOOD: usize = 10000;
const ANT_RADIUS: u16 = 1;
const FOOD_RADIUS: u16 = 1;
const WIDTH : f32 = 1500.0;
const HEIGHT : f32 = 900.0;
const HOME_X : f32 = WIDTH/2.0;
const HOME_Y : f32 = HEIGHT/2.0;
const HOME_RADIUS : f32 = 50.0;
const ANT_VISION : f32 = 150.0;
const PHEROMONE_DECAY : f32 = 0.009;

struct MainState {
    ants: ant::Ant,
    home: home::Home,
    food: food::Food,
    home_pheromones: Vec<Vec<f32>>,
    food_pheromones: Vec<Vec<f32>>,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = graphics::drawable_size(ctx);
        let home_position = na::Point2::new(HOME_X, HOME_Y);

        MainState {
            ants: ant::Ant::new(screen_w, screen_h),
            home: home::Home::new(home_position, HOME_RADIUS),
            food: food::Food::new(),
            home_pheromones: vec![vec![0.0; WIDTH as usize]; HEIGHT as usize],
            food_pheromones: vec![vec![0.0; WIDTH as usize]; HEIGHT as usize],
        }
    }

    pub fn initilize_positions(&mut self) {
        let mut init_data = ArrayVec::<(na::Point2<f32>, f32), TOTAL_ANTS>::new();
        for i in 0..TOTAL_ANTS {init_data.insert(i, self.home.generate_starting_position());}
        let init_data = init_data.into_inner().unwrap();
        self.ants.initialize(&init_data);
        
        self.food.add_food(na::Point2::new(50.0,50.0),
            na::Point2::new(WIDTH-50.0,100.0), TOTAL_FOOD as u32);
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        let ticks = ggez::timer::ticks(ctx);


        for i in 0..self.home_pheromones.len() {
            for j in 0..self.home_pheromones[0].len() {
                self.home_pheromones[i][j] -= PHEROMONE_DECAY;
                self.food_pheromones[i][j] -= PHEROMONE_DECAY;

                if self.home_pheromones[i][j] < 0.0 {
                    self.home_pheromones[i][j] = 0.0;
                }
                if self.food_pheromones[i][j] < 0.0 {
                    self.food_pheromones[i][j] = 0.0;
                }
            }
        }



        // self.ants.target_food_position.par_iter_mut()
        //     .zip(self.ants.target_food_index.par_iter_mut())
        //     .zip(self.ants.state.par_iter_mut())
        //     .zip(self.ants.velocity.par_iter_mut())
        //     .zip(self.ants.desired_direction.par_iter_mut())
        //     .for_each(|
        //         ((((food_position,
        //         food_index),
        //         ant_state),
        //         ant_velocity),
        //         ant_desired_direction)
        //     | {
        //         if 
        //     });



        for i in 0..TOTAL_ANTS {
            if self.ants.food_acquired(i) {
                self.ants.state[i] = 2;
                self.ants.set_antiparallel(i);
                self.ants.target_food_position[i] = None;
                match self.ants.target_food_index[i] {
                    None => (),
                    Some(j) => {
                        self.food.state[j as usize] = 2;
                        self.food.position[j as usize] = Some(
                            na::Point2::new(WIDTH + 2000.0, HEIGHT + 2000.0));
                    },
                }
            }


            if self.ants.state[i] == 0 {
                for j in 0..self.food.food_generated {
                    match self.food.position[j as usize] {
                        None => (),
                        Some(k) => {
                            if self.food.state[j as usize] == 0 && utils::distance(&k, &self.ants.position[i]) < ANT_VISION {
                                self.food.state[j as usize] = 1;
                                self.ants.set_food_target(i, k, j as u32);
                                break;
                            }
                        }
                    }
                }
            }

            if self.home.touching_home(self.ants.position[i]) {
                self.ants.state[i] = 0;
                self.ants.set_antiparallel(i);
            }


            if ticks % 1 == 0 {
                let approximate_y : usize = max(min(self.ants.position[i].y as usize, self.home_pheromones.len()-1), 0);
                let approximate_x : usize = max(min(self.ants.position[i].x as usize, self.home_pheromones[0].len()-1), 0);

                if self.ants.state[i] != 2 {
                    self.home_pheromones[approximate_y][approximate_x] = 1.0;
                } else {
                    self.food_pheromones[approximate_y][approximate_x] = 1.0;
                }
            }

            // Ant following pheromones algorithm

            let (home_angle, food_angle) = utils::ant_rays(self.ants.position[i], self.ants.angle[i], &self.home_pheromones, &self.food_pheromones);
            self.ants.set_pheromones_direction(i, home_angle, food_angle);
        }

        self.ants.update(dt);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        let draw_param = graphics::DrawParam::default();

        // let mut ant_sprite_batch = graphics::spritebatch::SpriteBatch::new(
        //     graphics::Image::solid(ctx, ANT_RADIUS, graphics::WHITE).unwrap()
        // );

        // for ant_index in 0..TOTAL_ANTS {
        //     ant_sprite_batch.add(graphics::DrawParam::new().dest(self.ants.position[ant_index]));
        // }

        // graphics::draw(ctx, &ant_sprite_batch, draw_param).unwrap();

        // draw food
        let mut food_sprite_batch = graphics::spritebatch::SpriteBatch::new(
            graphics::Image::solid(ctx, FOOD_RADIUS, graphics::Color::new(1.0, 0.0, 0.0, 1.0)).unwrap()
        );

        for i in 0..self.food.food_generated {
            match self.food.position[i as usize] {
                None => (),
                Some(j) => {
                    food_sprite_batch.add(graphics::DrawParam::new().dest(j));
                }
            }
        }

        graphics::draw(ctx, &food_sprite_batch, draw_param).unwrap();

        // draw pheromones

        let mut pheromones_sprite_batch = graphics::spritebatch::SpriteBatch::new(
            graphics::Image::solid(ctx, FOOD_RADIUS, graphics::Color::new(0.0, 1.0, 1.0, 1.0)).unwrap()
        );

        for y in 0..self.home_pheromones.len() {
            for x in 0..self.home_pheromones[0].len() {
                if self.home_pheromones[y][x] > 0.0 {
                    pheromones_sprite_batch.add(
                        graphics::DrawParam::new()
                            .dest(na::Point2::new(x as f32, y as f32))
                            .color(graphics::Color::new(0.0, 0.0, 1.0, self.home_pheromones[y][x]))
                    );
                }

                if self.food_pheromones[y][x] > 0.0 {
                    pheromones_sprite_batch.add(
                        graphics::DrawParam::new()
                            .dest(na::Point2::new(x as f32, y as f32))
                            .color(graphics::Color::new(0.0, 1.0, 0.0, self.food_pheromones[y][x]))
                    );
                }
            }
        }

        graphics::draw(ctx, &pheromones_sprite_batch, graphics::DrawParam::new()).unwrap();

        // ant colony
        let home_mesh = graphics::Mesh::new_circle(
            ctx, graphics::DrawMode::stroke(1.0),
            na::Point2::new(0.0,0.0),
            self.home.radius,
            graphics::FillOptions::DEFAULT_TOLERANCE,
            graphics::Color::from_rgb(87, 67, 227)).unwrap();

        graphics::draw(ctx, &home_mesh, graphics::DrawParam::new().dest(self.home.position)).unwrap();

        // Ant vision for pheromone

        // for i in 0..TOTAL_ANTS {
        //     let angle = self.ants.angle[i];
        //     let separation = 10.0;
        //     let vision_size = 20.0;
            
        //     let angles = [angle - PI/5.0, angle, angle + PI/5.0];
            
        //     for angle in angles.iter() {
        //         let x_0 = self.ants.position[i].x + (separation + vision_size) * angle.cos() - vision_size / 2.0;
        //         let y_0 = self.ants.position[i].y + (separation + vision_size) * angle.sin();
                
        //         let vision_box = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(1.0), graphics::Rect::new(x_0, y_0, vision_size, vision_size), graphics::WHITE).unwrap();
        //         graphics::draw(ctx, &vision_box, graphics::DrawParam::new()).unwrap();
        //     }
        // }

        // update the display
        graphics::present(ctx).unwrap();
        Ok(())
    }
}

fn main() {
    let window_mode = conf::WindowMode::default()
        .dimensions(WIDTH, HEIGHT);

    let cb = ContextBuilder::new("Ant Simulation", "Some One")
        .window_mode(window_mode);
    let (mut ctx, mut event_loop) = cb.build().unwrap();

    graphics::set_window_title(&ctx, "Ant Simulation");

    let mut state = MainState::new(&mut ctx);
    state.initilize_positions();

    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

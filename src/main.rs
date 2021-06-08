use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;
use ggez::graphics::{self};
use ggez::conf;
use std::f32;

mod ant;
mod utils;

const TOTAL_ANTS: usize = 5000;
const ANT_RADIUS: f32 = 1.0;
const WIDTH : f32 = 900.0;
const HEIGHT : f32 = 900.0;

struct MainState {
    ants: ant::Ant,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let (screen_w, screen_h) = graphics::drawable_size(ctx);
        let current_ants = ant::Ant::new(screen_w, screen_h);
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

        let current_ant_mesh = graphics::Mesh::new_ellipse(
            ctx, 
            graphics::DrawMode::fill(),
            na::Point2::new(0.0,0.0), ANT_RADIUS,
            ANT_RADIUS, 
            graphics::FillOptions::DEFAULT_TOLERANCE,
            graphics::WHITE).unwrap();

        for ant_index in 0..TOTAL_ANTS {
            draw_param.dest = self.ants.position[ant_index].into();
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
    
    let custom_window_config = conf::WindowMode {
        width: WIDTH,
        height: HEIGHT,
        ..Default::default()
    };

    graphics::set_mode(&mut ctx, custom_window_config).unwrap();
    graphics::set_window_title(&mut ctx, "Ant Simulation");

    let mut state = MainState::new(&mut ctx);
    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}
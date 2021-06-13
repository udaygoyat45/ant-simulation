use ggez::nalgebra as na;
use std::f32::consts::PI;

pub fn clamp_magnitude(x: &na::Vector2<f32>, c: f32) -> na::Vector2<f32> {
    x * (c / x.norm())
}

pub fn distance (a: &na::Point2<f32>, b: &na::Point2<f32>) -> f32 {
    return (a-b).norm();
}

pub fn index_calculator (boxes: &Vec<(f32, f32, f32, f32)>, pheromone_grid : &Vec<Vec<f32>>) -> (usize, f32) {
    let mut i = 0; 
    let mut h = -1.0;
    let mut hi = 0;

    for (x_0, y_0, x_1, y_1) in boxes {
        let mut current_score = 0.0;
        for y in (*y_0 as usize)..(*y_1 as usize) {
            for x in (*x_0 as usize)..(*x_1 as usize) {
                if y < pheromone_grid.len() && x < pheromone_grid[0].len() {
                    current_score += pheromone_grid[y][x];
                }
            }
        }

        if current_score > h {
            h = current_score;
            hi = i;
        }
        
        i += 1;
    }

    (hi, h)
}

pub fn ant_rays (position: na::Point2<f32>, angle : f32, home_pheromones_grid : &Vec<Vec<f32>>, food_pheromones_grid : &Vec<Vec<f32>>) -> (Option<f32>, Option<f32>) {
    let separation = 10.0;
    let vision_size = 20.0;

    let angles = [angle, angle - PI/5.0, angle + PI/5.0];
    let mut boxes = Vec::new();

    for angle in angles.iter() {
        let x_0 = position.x + (separation + vision_size) * angle.cos() - vision_size / 2.0;
        let y_0 = position.y + (separation + vision_size) * angle.sin();
        let x_1 = x_0 + vision_size;
        let y_1 = y_0 + vision_size;

        boxes.push((x_0, y_0, x_1, y_1));
    }

    let (home_index, home_score) = index_calculator(&boxes, home_pheromones_grid);
    let (food_index, food_score) = index_calculator(&boxes, food_pheromones_grid);
    // println!("{}, {}", food_index, food_score);

    let mut home_angle = None;
    let mut food_angle = None;

    if home_score > 0.0 {
        home_angle = Some(angles[home_index]);
    }

    if food_score > 0.0 {
        food_angle = Some(angles[food_index]);
    }

    (home_angle, food_angle)
}
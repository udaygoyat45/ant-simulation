use ggez::nalgebra as na;

pub fn clamp_magnitude(x: &na::Vector2<f32>, c: f32) -> na::Vector2<f32> {
    x * (c / x.norm())
}

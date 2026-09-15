use macroquad::prelude::glam;
use glam::*;

pub fn get_rot_mat(a: f32) -> Mat2 {
    mat2(vec2(a.cos(), a.sin()), vec2(-a.sin(), a.cos()))
}

pub fn format_float(val: f32, precision: i32) -> String {
    if precision >= 0 {
        format!("{:.1$}", val, precision as usize)
    } else {
        let factor = 10f32.powi(-precision);
        let rounded = (val / factor).round() * factor;
        format!("{:.0}", rounded)
    }
}

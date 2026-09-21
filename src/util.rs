use macroquad::prelude::glam::*;
use std::collections::BTreeMap;

pub fn btreemap_usize_get_unused_id<T>(map: &BTreeMap<usize, T>) -> usize{
    map.last_key_value().map(|(&k, _)|{k + 1}).unwrap_or(0)
}

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

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

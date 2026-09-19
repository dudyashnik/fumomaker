use std::f32::consts::PI;
use macroquad::prelude::glam::*;
use macroquad::color::*;
use macroquad::prelude::{draw_text_ex, TextParams};
use macroquad::shapes::*;
use macroquad::window::*;
use macroquad::text::*;
use macroquad::input::*;
use crate::util::*;

const DOT_RADIUS: f32 = 6.;

pub fn is_ctrl() -> bool {
    is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
}

pub fn is_shift() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

pub fn is_alt() -> bool {
    is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt)
}

pub fn is_no_mod_down() -> bool {
    !is_shift() && !is_ctrl() && !is_alt()
}

pub fn is_pressed_with_shift(key_code: KeyCode) -> bool {
    is_key_pressed(key_code) && is_shift() && !is_ctrl() && !is_shift()
}

pub fn is_pressed_with_ctrl(key_code: KeyCode) -> bool {
    is_key_pressed(key_code) && !is_shift() && is_ctrl() && !is_shift()
}

pub fn is_pressed_with_alt(key_code: KeyCode) -> bool {
    is_key_pressed(key_code) && !is_shift() && !is_ctrl() && is_shift()
}

pub fn is_pressed_with_no_mod(key_code: KeyCode) -> bool {
    is_key_pressed(key_code) && !is_ctrl() && !is_shift() && !is_alt()
}

pub fn get_mouse_position_vec2() -> Vec2 {
    vec2(mouse_position().0, mouse_position().1)
}

pub fn is_mouse_near_dot(sp: Vec2) -> bool {
    (get_mouse_position_vec2() - sp).length() < DOT_RADIUS
}

pub fn get_different_gray_color(c: Vec3) -> Vec3 {
    let intensity = 0.21 * c.x + 0.71 * c.y + 0.08 * c.z;
    let d = (|x| { if (x < 0.5) {1.0} else {0.0} })(intensity);
    vec3(d, d, d)
}

pub fn vec3_to_mq_color(c: Vec3) -> Color {
    Color::new(c.x, c.y, c.z, 1.0)
}


pub fn draw_my_font(font: &Font, txt: &str, x: f32, y: f32, color: Color, font_size: u16) {
    draw_text_ex(txt, x, y, TextParams {color, font: Some(&font),
        font_size, font_scale: 1f32, ..Default::default()});
}

pub fn draw_rectangle_with_borders(px: f32, py: f32, width: f32, height: f32, border_clr: Color, clr: Color){
    let ob = 1.0;
    draw_rectangle_lines(px - ob, py - ob, width + ob * 2f32, height + ob * 2f32, ob * 2f32, border_clr);
    draw_rectangle(px, py, width, height, clr);
}

pub fn draw_color_descr_square(px: f32, py: f32, dim: f32, color: Vec3){
    let ob = 1.0;
    draw_rectangle_lines(px - ob, py - ob, dim + ob * 2f32, dim + ob * 2f32,
                         ob * 2f32, vec3_to_mq_color(get_different_gray_color(color)));
    draw_rectangle(px, py, dim, dim, Color::new(color.x, color.y, color.z, 1.0));
}


#[derive(Clone)]
pub struct Camera {
    pub px_in_mm: f32,
    pub top_left: Vec2,
}

impl Camera {
    // Returns scene space coordinates
    pub fn get_mouse_scene_pos(&self) -> Vec2 {
        self.screen_coord_to_scene(get_mouse_position_vec2())
    }

    // Takes scene space coordinates
    pub fn is_mouse_near_scene_dot(&self, v: Vec2) -> bool {
        (self.scene_coord_to_screen(v) - get_mouse_position_vec2()).length() < DOT_RADIUS
    }

    pub fn scene_coord_to_screen(&self, scene_c: Vec2) -> Vec2{
        let n = scene_c - self.top_left;
        vec2(n.x * self.px_in_mm, -n.y * self.px_in_mm)
    }

    pub fn screen_coord_to_scene(&self, screen_c: Vec2) -> Vec2{
        self.top_left + vec2(screen_c.x / self.px_in_mm, -screen_c.y / self.px_in_mm)
    }

    // Takes scene space coordinates
    pub fn draw_mq_line_on_scene(&self, a: Vec2, b: Vec2, thickness: f32, color: Color) {
        let sa = self.scene_coord_to_screen(a);
        let sb = self.scene_coord_to_screen(b);
        draw_line(sa.x, sa.y, sb.x, sb.y, thickness, color)
    }

    // Takes screen space coordinates
    pub fn draw_arrow_on_screen(sa: Vec2, sb: Vec2, thickness: f32, color: Color){
        let back = (-sb + sa).normalize() * 16.;
        let p1 = sb + get_rot_mat(PI * 0.25) * back;
        let p2 = sb + get_rot_mat(-PI * 0.25) * back;
        draw_line(sa.x, sa.y, sb.x, sb.y, thickness, color);
        draw_line(p1.x, p1.y, sb.x, sb.y, thickness, color);
        draw_line(p2.x, p2.y, sb.x, sb.y, thickness, color);
    }

    // Takes scene space coordinates
    pub fn draw_arrow(&self, a: Vec2, b: Vec2, thickness: f32, color: Color){
        Self::draw_arrow_on_screen(self.scene_coord_to_screen(a), self.scene_coord_to_screen(b), thickness, color);
    }

    pub fn draw_geom_segment(&self, a: Vec2, b: Vec2, thickness: f32, break_thickness : f32, color: Color) {
        let sa = self.scene_coord_to_screen(a);
        let sb = self.scene_coord_to_screen(b);
        let par = (-sb + sa).normalize();
        let perp = vec2(par.y, par.x) * f32::min(thickness, (-sb + sa).length());
        let p1 = sa + perp;
        let p2 = sa - perp;
        let p3 = sb + perp;
        let p4 = sb - perp;
        draw_line(sa.x, sa.y, sb.x, sb.y, thickness, color);
        draw_line(p1.x, p1.y, p2.x, p2.y, break_thickness, color);
        draw_line(p3.x, p3.y, p4.x, p4.y, break_thickness, color);
    }

    pub fn draw_circle_on_screen(a: Vec2, rad: f32, color: Color){
        draw_circle(a.x, a.y, rad, color);
    }

    // Takes scene coordinates
    pub fn draw_circle(&self, a: Vec2, rad: f32, color: Color){
        Self::draw_circle_on_screen(self.scene_coord_to_screen(a), rad, color);
    }

    pub fn draw_circle_with_perimeter_on_screen(s: Vec2, rad: f32, inner_color: Color, thickness: f32, perimeter_color: Color){
        draw_circle(s.x, s.y, rad, inner_color);
        draw_circle_lines(s.x, s.y, rad + thickness, thickness, perimeter_color);
    }

    // Takes scene space coordinates
    pub fn draw_circle_with_perimeter(&self, a: Vec2, rad: f32, inner_color: Color, thickness: f32, perimeter_color: Color) {
        Self::draw_circle_with_perimeter_on_screen(self.scene_coord_to_screen(a), rad, inner_color, thickness, perimeter_color);
    }

    pub fn draw_decor_square_on_screen(s: Vec2, rad: f32, inner_color: Color, thickness: f32, perimeter_color: Color){
        draw_rectangle(s.x - rad, s.y - rad, rad * 2., rad * 2., inner_color);
        draw_rectangle_lines(s.x - rad - thickness, s.y - rad - thickness,
                             (rad + thickness) * 2., (rad + thickness) * 2., thickness, perimeter_color);
    }

    // Takes scene space coordinates
    pub fn draw_decor_square(&self, a: Vec2, rad: f32, inner_color: Color, thickness: f32, perimeter_color: Color){
        Self::draw_decor_square_on_screen(self.scene_coord_to_screen(a), rad, inner_color, thickness, perimeter_color);
    }

    pub fn draw_dash_line_on_screen(sa: Vec2, sb: Vec2, px_offset: f32, thickness: f32,
                                    worm_length: f32, worm_tail_dist: f32, color: Color){
        let l_full = (-sa+sb).length();
        let mut a = ((sa.x as f64 * 256.0).round() as i64, (sa.y as f64 * 256.0).round() as i64);
        let mut b = ((sb.x as f64 * 256.0).round() as i64, (sb.y as f64 * 256.0).round() as i64);
        let mut w = (screen_width() as f64 * 256.0).round() as i64;
        let mut h = (screen_height() as f64 * 256.0).round() as i64;

        let mut l_start: f32 = 0.;
        let mut l_end: f32 = l_full;

        if a.0 < b.0 {
            l_start = f32::max(l_start, -10.0 + ((0 - a.0) as f64 / 256.) as f32);
            l_end = f32::min(l_end, 10. + ((w - a.0) as f64 / 256.) as f32);
        } else if b.0 < a.0 {
            l_start = f32::max(l_start, -10.0 + (((-w + a.0) as f64 / 256.) as f32));
            l_end = f32::min(l_end, 10. + (((-0 + a.0) as f64 / 256.) as f32));
        } else if a.1 < -1000 || a.1 > h + 1000 {
            l_end = -1.;
        }

        if a.1 < b.1 {
            l_start = f32::max(l_start, -10.0 + ((0 - a.1) as f64 / 256.) as f32);
            l_end = f32::min(l_end, 10. + ((h - a.1) as f64 / 256.) as f32);
        } else if b.1 < a.1 {
            l_start = f32::max(l_start, -10.0 + (((-h + a.1) as f64 / 256.) as f32));
            l_end = f32::min(l_end, 10. + (((-0 + a.1) as f64 / 256.) as f32));
        } else if a.0 < -1000 || a.0 > w + 1000 {
            l_end = -1.;
        }

        if l_start > l_end { return }

        let mut cur: f32 = - px_offset + ((px_offset + l_start) / worm_length).floor() * worm_length;
        while cur < l_end {
            let cur_end = cur + worm_length;
            let w1 = sa + (sb - sa).normalize() * cur;
            let w2 = sa + (sb - sa).normalize() * cur_end;
            draw_line(w1.x, w1.y, w2.x, w2.y, thickness, color);
            cur += worm_tail_dist;
        }
    }

    // takes scene space coordinates
    pub fn draw_dash_line(&self, scene_a: Vec2, scene_b: Vec2, px_offset: f32, thickness: f32,
                          worm_length: f32, worm_tail_dist: f32, color: Color){
        Self::draw_dash_line_on_screen(
            self.scene_coord_to_screen(scene_a), self.scene_coord_to_screen(scene_b),
            px_offset, thickness, worm_length, worm_tail_dist, color);
    }
}


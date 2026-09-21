use std::f32::consts::PI;
use enum_as_inner::EnumAsInner;
use macroquad::prelude::glam::*;
use macroquad::color::*;
use macroquad::prelude::{draw_text_ex, TextParams};
use macroquad::shapes::*;
use macroquad::window::*;
use macroquad::text::*;
use macroquad::input::*;
use crate::util::*;

pub const COLOR_VEC_BLACK: Vec3 = vec3(0., 0., 0.);
pub const COLOR_VEC_GRAY: Vec3 = vec3(0.5, 0.5, 0.5);
pub const COLOR_VEC_WHITE: Vec3 = vec3(1., 1., 1.);
pub const COLOR_VEC_ORANGE: Vec3 = vec3(0.9, 0.5, 0.);
pub const COLOR_VEC_BLUE: Vec3 = vec3(0.1, 0.2, 0.95);
pub const COLOR_VEC_LIGHT_BLUE: Vec3 = vec3(0.5, 0.6, 1.);
pub const COLOR_VEC_PURPLE: Vec3 = vec3(0.8, 0., 1.);
pub const COLOR_VEC_BLUE_2: Vec3 = vec3(0.0, 0.5, 1.);

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
    is_key_pressed(key_code) && is_shift() && !is_ctrl() && !is_alt()
}

pub fn is_pressed_with_ctrl(key_code: KeyCode) -> bool {
    is_key_pressed(key_code) && !is_shift() && is_ctrl() && !is_alt()
}

pub fn is_pressed_with_alt(key_code: KeyCode) -> bool {
    is_key_pressed(key_code) && !is_shift() && !is_ctrl() && is_alt()
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

// a, b, c are screen coordinates
pub fn dist_to_segment_on_screen(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let ab = b - a;
    let ac = c - a;
    let len_sq = ab.dot(ab);
    let t = if len_sq == 0.0 { 0.0 } else { (ac.dot(ab) / len_sq).clamp(0.0, 1.0) };
    let closest = a + ab * t;
    (c - closest).length()
}

#[derive(EnumAsInner, Copy, Clone)]
enum DrawnPathStyle {
    None,
    Dashed(Color),
    Plain(Color),
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
        let l = (-sb + sa).length();
        let back = (-sb + sa).normalize() * (if l > 32. {16.} else {l / 2.});
        let p1 = sb + get_rot_mat(PI * 0.12) * back;
        let p2 = sb + get_rot_mat(-PI * 0.12) * back;
        draw_line(sa.x, sa.y, sb.x, sb.y, thickness, color);
        draw_triangle(p1, p2, sb, color);
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
        let l_full = (sb - sa).length();
        if l_full == 0. || worm_length <= 0. || worm_tail_dist <= 0. {
            return
        }

        let direction = (sb - sa) / l_full;
        const CLIP_MARGIN: f32 = 10.;

        let mut l_start: f32 = 0.;
        let mut l_end: f32 = l_full;

        for (start, delta, limit) in [
            (sa.x, direction.x, screen_width()),
            (sa.y, direction.y, screen_height()),
        ] {
            if delta.abs() < f32::EPSILON {
                if start < -CLIP_MARGIN || start > limit + CLIP_MARGIN {
                    return
                }
                continue
            }

            let d1 = (-CLIP_MARGIN - start) / delta;
            let d2 = (limit + CLIP_MARGIN - start) / delta;
            l_start = l_start.max(d1.min(d2));
            l_end = l_end.min(d1.max(d2));
        }

        if l_start > l_end { return }

        let first_worm = ((l_start + px_offset - worm_length) / worm_tail_dist).ceil();
        let mut cur = first_worm * worm_tail_dist - px_offset;
        while cur < l_end {
            let cur_start = cur.max(l_start);
            let cur_end = (cur + worm_length).min(l_end);
            if cur_start < cur_end {
                let w1 = sa + direction * cur_start;
                let w2 = sa + direction * cur_end;
                draw_line(w1.x, w1.y, w2.x, w2.y, thickness, color);
            }
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

    pub fn draw_dashed_path_on_screen(p: &[Vec2], px_offset: f32, thickness: f32,
                            worm_length: f32, worm_tail_dist: f32, color: Color){
        let mut lb: f32 = px_offset;
        for i in 1..p.len() {
            let a = p[i - 1];
            let b = p[i];
            Camera::draw_dash_line_on_screen(a, b, lb, thickness, worm_length, worm_tail_dist, color);
            lb += (-a + b).length();
        }
    }

    pub fn draw_dashed_contour_on_screen(p: &[Vec2], px_offset: f32, thickness: f32,
                 worm_length: f32, worm_tail_dist: f32, color: Color, closed: bool){
        let mut lb: f32 = px_offset;
        for i in (if closed {1..p.len() + 1} else {1..p.len()}) {
            let a = p[i - 1];
            let b = p[if i >= p.len() { 0 } else { i }];
            Camera::draw_dash_line_on_screen(a, b, lb, thickness, worm_length, worm_tail_dist, color);
            lb += (-a + b).length();
        }
    }

    pub fn draw_path_on_screen(p: &[Vec2], thickness: f32, color: Color){
        for i in 1..p.len() {
            let a = p[i - 1];
            let b = p[i];
            draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }
    }

    pub fn draw_contour_on_screen(p: &[Vec2], thickness: f32, color: Color, closed: bool){
        for i in (if closed {1..p.len() + 1} else {1..p.len()}) {
            let a = p[i - 1];
            let b = p[if i >= p.len() { 0 } else { i }];
            draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }
    }

    // (a & b) are in scene space coordinates. sc is in screen coordinates
    pub fn distance_to_segment(&self, sc: Vec2, a: Vec2, b: Vec2) -> f32 {
        dist_to_segment_on_screen(self.scene_coord_to_screen(a), self.scene_coord_to_screen(b), sc)
    }

    // segment points a, b are in scene space coordinates
    pub fn is_mouse_near_segment(&self, a: Vec2, b: Vec2) -> bool {
        self.distance_to_segment(get_mouse_position_vec2(), a, b) < DOT_RADIUS
    }

    pub fn is_mouse_near_path(&self, path: &[Vec2]) -> bool {
        for i in 1..path.len(){
            let a = path[i - 1];
            let b = path[i];
            if self.distance_to_segment(get_mouse_position_vec2(), a, b) < DOT_RADIUS {
                return true;
            }
        }
        return false;
    }

    pub fn to_which_path_segment_we_are_close(&self, path: &[Vec2]) -> Option<usize> {
        for i in 1..path.len(){
            let a = path[i - 1];
            let b = path[i];
            if self.distance_to_segment(get_mouse_position_vec2(), a, b) < DOT_RADIUS {
                return Some(i - 1);
            }
        }
        return None;
    }
}

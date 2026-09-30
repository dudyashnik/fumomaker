#![feature(int_roundings)]

mod scene;
mod embroidery;
mod util;
mod editor_visual;
mod drawing;
mod editor;
mod files;
mod trenches;

use std::cmp::min;
use macroquad::window::*;
use macroquad::color::*;
use macroquad::texture::*;
use macroquad::shapes::*;
use macroquad::text::*;
use macroquad::input::*;

use macroquad::prelude::glam;
use glam::f32::*;

use std::default::Default;
use std::env::args;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::f32::consts::PI;
use std::time::Instant;
use std::fs;
use enum_as_inner::EnumAsInner;

use util::*;
use scene::*;
use embroidery::*;
use drawing::*;
use editor_visual::*;
use editor::*;
use files::*;

#[derive(Clone)]
struct CoolColor {
    clr: Vec3,
    name: &'static str,
}

const COOL_COLORS: &[CoolColor] = &[
    CoolColor{clr: Vec3::new(0.7, 0.0, 0.0), name: "dark red"},
    CoolColor{clr: Vec3::new(1.0, 0.0, 0.0), name: "red"},
    CoolColor{clr: Vec3::new(0.0, 1.0, 0.0), name: "light green"},
    CoolColor{clr: Vec3::new(0.0, 0.6, 0.0), name: "green"},
    CoolColor{clr: Vec3::new(0.0, 0.0, 1.0), name: "blue"},
    CoolColor{clr: Vec3::new(0.5, 0.5, 1.0), name: "light blue"},
    CoolColor{clr: Vec3::new(0.0, 0.0, 0.0), name: "black"},
    CoolColor{clr: Vec3::new(0.3, 0.3, 0.3), name: "gray"},
    CoolColor{clr: Vec3::new(0.7, 0.7, 0.7), name: "light gray"},
    CoolColor{clr: Vec3::new(1., 0.753, 0.796), name: "pink"},
    CoolColor{clr: Vec3::new(1., 1., 0.), name: "yellow"},
];


impl From<&CoolColor> for UsedColor {
    fn from(c: &CoolColor) -> Self {
        Self {clr: c.clr, name: String::from(c.name)}
    }
}


#[macroquad::main("Texture")]
async fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        println!("Usage: fumomaker <scene_file.fm_scene>");
        return;
    }
    let scene_file_name: String = args[1].clone();
    let embroidery_file_name: String = scene_file_name.clone() + ".emb.json";
    let font = load_ttf_font("src/fonts/GreatVibes-Regular.ttf").await.expect("Can't load font");

    let mut last_error_message: String = "".to_string();

    let mut editor = {
        let scene = if let Ok(q) = fs::exists(&scene_file_name) && q {
            load_scene_from_file(&scene_file_name).unwrap_or_else(|err| {
                last_error_message = err.to_string();
                Scene::default()
            })
        } else {
            Scene::default()
        };
        Editor::new(scene)
    };

    prevent_quit();
    let t_start = Instant::now();
    let mut t_previous = t_start;
    loop {
        let t_now = Instant::now();
        clear_background(vec3_to_mq_color(editor.scene.get_bg_color()));
        editor.draw_grid(&font);
        editor.draw_scene((t_now - t_start).as_secs_f32());
        editor.draw_info_label(&font, &embroidery_file_name, &last_error_message);
        editor.draw_color_list(&font);
        editor.draw_node_list(&font);
        editor.draw_full_info_text(&font);

        if is_pressed_with_ctrl(KeyCode::S) {
            if let Err(err) = save_scene_to_file(&scene_file_name, &editor.scene) {
                last_error_message = err.to_string();
            }
        }

        if is_pressed_with_ctrl(KeyCode::E){
            let img = match &editor.stitches {
                Some(img) => img,
                None => &build_embroidery_image(&editor.scene)
            };
            if let Err(err) = save_embroidery_image_to_file(&embroidery_file_name, img){
                last_error_message = err.to_string();
            }
        }

        if is_mouse_button_pressed(MouseButton::Right) && !is_shift() && is_ctrl() && is_alt() {
            editor.command_click_delete_vertex_of_selected_shape();
        }

        if is_mouse_button_pressed(MouseButton::Left) && !is_shift() && is_ctrl() && is_alt() {
            editor.command_click_add_vertex_to_selected_shape();
        }

        if is_mouse_button_pressed(MouseButton::Left) && is_no_mod_down() {
            editor.command_drawing_mode_click_normal();
        }

        if is_mouse_button_pressed(MouseButton::Left) && is_shift() {
            editor.command_drawing_mode_click_negative();
        }

        // Requires release event for this mouse button
        if is_mouse_button_pressed(MouseButton::Left) && !is_shift() && is_ctrl() && !is_alt() {
            editor.command_click_select_vertex_of_selected_shape_for_moving_vert();
        }

        // Requires release event for this mouse button
        if is_mouse_button_pressed(MouseButton::Left) && is_no_mod_down(){
            editor.command_embroidery_config_control_click_normal();
        }

        if is_mouse_button_released(MouseButton::Left) {
            editor.command_embroidery_config_control_pointer_btn_release();
            editor.command_moving_vertex_pointer_btn_release();
        }

        if is_mouse_button_pressed(MouseButton::Right) && is_no_mod_down(){
            editor.command_drawing_mode_selection_click();
        }

        editor.command_ack_pointer_motion();

        if is_pressed_with_no_mod(KeyCode::Escape){
            editor.command_cancellation();
        }

        if is_pressed_with_no_mod(KeyCode::Z) {
            editor.command_finish_drawing_progress();
        }

        if is_pressed_with_no_mod(KeyCode::E){
            editor.command_set_editor_mode_embroidery_config();
        }
        if is_pressed_with_no_mod(KeyCode::N){
            editor.command_set_editor_mode_drawing();
        }
        if is_pressed_with_shift(KeyCode::C) {
            editor.command_set_editor_mode_stitch_image_viewing();
        }
        if is_pressed_with_no_mod(KeyCode::Escape) && editor.mode.is_stitch() {
            editor.command_set_editor_mode_embroidery_config();
        }
        if is_pressed_with_no_mod(KeyCode::Key1){
            editor.command_change_tool_to_area();
        }
        if is_pressed_with_no_mod(KeyCode::Key2){
            editor.command_change_tool_to_thin_line();
        }
        if is_pressed_with_no_mod(KeyCode::Key3){
            editor.command_change_tool_to_thick_line();
        }
        if is_pressed_with_no_mod(KeyCode::Key4){
            editor.command_change_tool_to_sym();
        }
        if is_pressed_with_no_mod(KeyCode::P){
            editor.perform_select_shape_for_reparenting()
        }
        if is_pressed_with_no_mod(KeyCode::H){
            editor.perform_selected_selected_for_movement_bind();
        }
        if is_pressed_with_no_mod(KeyCode::G) {
            editor.perform_attaching_shape_to_group();
        }
        if is_pressed_with_no_mod(KeyCode::O){
            editor.command_put_object_in_its_own_area_group();
        }
        if is_pressed_with_no_mod(KeyCode::M){
            editor.command_create_a_ghost_object();
        }

        if is_pressed_with_alt(KeyCode::Up){
            editor.command_select_previous_color();
        }
        if is_pressed_with_alt(KeyCode::Down){
            editor.command_select_next_color();
        }

        if is_pressed_with_no_mod(KeyCode::Delete) {
            editor.command_delete_selected();
        }
        if is_pressed_with_shift(KeyCode::Delete){
            editor.command_delete_selected_suppress_ghost_recoil();
        }

        if is_pressed_with_no_mod(KeyCode::Left){
            editor.command_stitch_image_viewing_progress_back(1);
        }
        if is_pressed_with_ctrl(KeyCode::Left){
            editor.command_stitch_image_viewing_progress_back(10);
        }
        if is_pressed_with_alt(KeyCode::Left){
            editor.command_stitch_image_viewing_progress_back(100);
        }

        if is_pressed_with_no_mod(KeyCode::Right){
            editor.command_stitch_image_viewing_progress_forward(1);
        }
        if is_pressed_with_ctrl(KeyCode::Right){
            editor.command_stitch_image_viewing_progress_forward(10);
        }
        if is_pressed_with_alt(KeyCode::Right){
            editor.command_stitch_image_viewing_progress_forward(100);
        }

        if is_pressed_with_no_mod(KeyCode::T){
            editor.command_toggle_some_param_of_selected_shape();
        }
        if is_pressed_with_no_mod(KeyCode::B){
            editor.command_remove_last_vertex_of_edited_path();
        }
        if is_pressed_with_alt(KeyCode::C) {
            editor.command_recolor_selected_object();
        }

        editor.command_ack_pointer_motion();
        editor.command_ack_mouse_wheel_motion();

        if mouse_wheel().1 > 0. && is_ctrl() && !is_shift() && !is_alt() {
            editor.command_increase_gap_primal_trench_gap_of_area_shape();
        }

        if mouse_wheel().1 < 0. && is_ctrl() && !is_shift() && !is_alt() {
            editor.command_decrease_gap_primal_trench_gap_of_area_shape();
        }

        if is_no_mod_down(){
            editor.command_ack_pressed_motion_keys(t_previous, t_now,
                                                   is_key_down(KeyCode::W), is_key_down(KeyCode::A),
                                                   is_key_down(KeyCode::S), is_key_down(KeyCode::D));
        }

        if is_quit_requested(){
            if let Err(err) = save_scene_to_file(&scene_file_name, &editor.scene) {
                println!("{}", err.to_string());
            }
            break;

        }
        next_frame().await;
        t_previous = t_now;
    }
}

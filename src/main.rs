#![feature(int_roundings)]

mod scene;
mod embroidery;
mod util;
mod editor_visual;
mod drawing;
mod editor;
mod files;

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
];


impl From<&CoolColor> for UsedColor {
    fn from(c: &CoolColor) -> Self {
        Self {clr: c.clr, name: String::from(c.name)}
    }
}


#[macroquad::main("Texture")]
async fn main() {
    println!("1123123123");
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

        if is_pressed_with_ctrl(KeyCode::S) {
            if let Err(err) = save_scene_to_file(&scene_file_name, &editor.scene) {
                last_error_message = err.to_string();
            }
        }

        if is_pressed_with_ctrl(KeyCode::E){
            if let Some(img) = &editor.stitches {
                if let Err(err) = save_embroidery_image_to_file(&embroidery_file_name, img){
                    last_error_message = err.to_string();
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) && is_no_mod_down(){
            editor.command_embroidery_config_control_click_normal();
        }

        if is_mouse_button_pressed(MouseButton::Left) && is_no_mod_down() {
            editor.command_drawing_mode_click_normal();
        }

        if is_mouse_button_pressed(MouseButton::Left) && is_shift() {
            editor.command_drawing_mode_click_negative();
        }

        if is_mouse_button_released(MouseButton::Left) {
            editor.command_embroidery_config_control_pointer_btn_release();
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
            editor.stitches = None;
            editor.mode = EditorMode::Embroidery;
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

        editor.command_ack_pointer_motion();
        editor.command_ack_mouse_wheel_motion();

        if is_no_mod_down(){
            editor.command_ack_pressed_motion_keys(t_previous, t_now,
                                                   is_key_down(KeyCode::W), is_key_down(KeyCode::A),
                                                   is_key_down(KeyCode::S), is_key_down(KeyCode::D));
        }



        if is_quit_requested(){
            // todo: save
            break;
        }
        next_frame().await;
        t_previous = t_now;
    }
}

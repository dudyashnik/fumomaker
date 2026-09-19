#![feature(int_roundings)]

mod scene;
mod embroidery;
mod util;
mod editor_visual;
mod drawing;
mod editor;

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
use enum_as_inner::EnumAsInner;

use util::*;
use scene::*;
use embroidery::*;
use drawing::*;
use editor_visual::*;
use editor::*;

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
    let embroidery_file_name: String = scene_file_name + ".emb.json";
    let font = load_ttf_font("src/fonts/GreatVibes-Regular.ttf").await.expect("Can't load font");

    let mut editor = Editor::new(Scene::default());

    editor.scene.colors.insert(0, UsedColor::from(&COOL_COLORS[0]));
    editor.scene.colors.insert(1, UsedColor::from(&COOL_COLORS[1]));
    editor.scene.colors.insert(2, UsedColor::from(&COOL_COLORS[2]));
    editor.selected_color = Some(1);
    editor.scene.movements.insert(0, MovementNode::SymmetryMovement(SymmetryMovement{x: 110.0, pos_y1: 2.0, pos_y2: 30.0}));
    editor.scene.movements.insert(1, MovementNode::SymmetryMovement(SymmetryMovement{x: -110.0, pos_y1: 2.0, pos_y2: 30.0}));
    editor.selected_movement = Some(0);
    editor.scene.objects.insert(0, GroupedObjectNode{obj: ObjectNode::GhostObject(GhostObject{movement: 1, source: 1 }), group: Some(0)});
    editor.scene.objects.insert(1, GroupedObjectNode{obj: ObjectNode::RealObjectNode(
        RealObjectNode{att: RealObjectAttrs{shape: Shape::AreaShape(AreaShape{
            points: vec![vec2(10.0, 10.0), vec2(100.0, 10.0), vec2(10.0, 100.0)], is_gap: false
        }), color: 0}, clone: Some(0)}), group: Some(1)});
    editor.scene.objects.insert(2, GroupedObjectNode{obj: ObjectNode::GhostObject(GhostObject{movement: 1, source: 3 }), group: Some(2)});
    editor.scene.objects.insert(3, GroupedObjectNode{obj: ObjectNode::RealObjectNode(
        RealObjectNode{att: RealObjectAttrs{shape: Shape::AreaShape(AreaShape{
            points: vec![vec2(10.0, 210.0), vec2(100.0, 210.0), vec2(10.0, 300.0)], is_gap: false
        }), color: 2}, clone: Some(2)}), group: Some(2)});
    editor.scene.area_groups.insert(0, AreaShapeGroup{control_center_pos: vec2(30.0, 30.0),
        control_fill_dir_offset: vec2(90.0, 0.0), f_params: AreaDoubleFillParams::default(),
        perimeters: BTreeSet::from([0])});
    editor.scene.area_groups.insert(1, AreaShapeGroup{control_center_pos: vec2(-100.0, 30.0),
        control_fill_dir_offset: vec2(90.0, 0.0), f_params: AreaDoubleFillParams::default(),
        perimeters: BTreeSet::from([1])});
    editor.scene.area_groups.insert(2, AreaShapeGroup{control_center_pos: vec2(0.0, 120.0),
        control_fill_dir_offset: vec2(90.0, 0.0), f_params: AreaDoubleFillParams::default(),
        perimeters: BTreeSet::from([2, 3])});

    prevent_quit();
    let t_start = Instant::now();
    let mut t_previous = t_start;
    let mut t_started_moving: Option<Instant> = None;
    loop {
        let t_now = Instant::now();
        clear_background(vec3_to_mq_color(editor.scene.get_bg_color()));
        editor.draw_grid(&font);
        editor.draw_scene();
        editor.draw_info_label(&font, &embroidery_file_name);
        editor.draw_color_list(&font);
        editor.draw_node_list(&font);

        if is_mouse_button_pressed(MouseButton::Left){
            (|| {
                if editor.mode.is_embroidery(){
                    for (&group_id, group) in &editor.scene.area_groups {
                        let (a, b) = editor.get_area_group_control_arrow_scr_pos(group);
                        if is_mouse_near_dot(a) {
                            editor.held_arrow_start_embroidery_control_group = Some(group_id);
                            return;
                        }
                        if is_mouse_near_dot(b) {
                            editor.held_arrow_end_embroidery_control_group = Some(group_id);
                            return;
                        }
                    }
                } else if editor.mode.is_draw() {
                    let p = editor.cam.get_mouse_scene_pos();
                    match editor.selected_tool {
                        EditorDrawingTool::Symmetry => {
                            if editor.held_edited_sym.is_some(){
                                editor.held_edited_sym = None;
                            } else {
                                let id = btreemap_usize_get_unused_id(&editor.scene.movements);
                                editor.scene.movements.insert(id, MovementNode::SymmetryMovement(SymmetryMovement{
                                    x: p.x, pos_y1: p.y, pos_y2: p.y
                                }));
                                editor.held_edited_sym = Some(id);

                            }
                        }
                        EditorDrawingTool::Area => {
                            if let Some(cur_edited_id) = editor.held_edited_perimeter {
                                let first = *editor.get_points_mut_of_area(cur_edited_id).first().unwrap();
                                if editor.cam.is_mouse_near_scene_dot(first) {
                                    editor.get_points_mut_of_area(cur_edited_id).pop();
                                    editor.held_edited_perimeter = None;
                                } else {
                                    let last = *editor.get_points_mut_of_area(cur_edited_id).last().unwrap();
                                    editor.get_points_mut_of_area(cur_edited_id).push(last);
                                }
                            } else {
                                let working_color_id = match editor.selected_color { Some(x) => x, None => return };
                                let is_gap = is_shift();
                                let id = btreemap_usize_get_unused_id(&editor.scene.objects);
                                let new_group_id = btreemap_usize_get_unused_id(&editor.scene.area_groups);
                                editor.scene.area_groups.insert(new_group_id, AreaShapeGroup{
                                    control_center_pos: p, control_fill_dir_offset: vec2(1., 0.),
                                    f_params : AreaDoubleFillParams::default(),
                                    perimeters: BTreeSet::from([id])
                                });
                                editor.scene.objects.insert(id, GroupedObjectNode{
                                    obj: ObjectNode::RealObjectNode(RealObjectNode{
                                        att: RealObjectAttrs{ shape: Shape::AreaShape(AreaShape {
                                            points: vec![p, p], is_gap
                                        }), color: working_color_id },
                                        clone: None
                                    }), group: Some(new_group_id)});
                                editor.held_edited_perimeter = Some(id);
                            }
                        }
                        EditorDrawingTool::ThinLine => {
                            if let Some(edited_line_id) = editor.held_edited_thin_line {
                                let last = *editor.get_points_mut_of_thin_line(edited_line_id).last().unwrap();
                                editor.get_points_mut_of_thin_line(edited_line_id).push(last);
                            } else {
                                let working_color_id = match editor.selected_color { Some(x) => x, None => return };
                                let id = btreemap_usize_get_unused_id(&editor.scene.objects);
                                editor.scene.objects.insert(id, GroupedObjectNode {group: None,
                                    obj: ObjectNode::RealObjectNode(RealObjectNode {clone: None,
                                        att: RealObjectAttrs {
                                            shape: Shape::LineShape(LineShape{points: vec![p, p]}),
                                            color: working_color_id }
                                    })});
                                editor.held_edited_thin_line = Some(id);
                            }
                        }
                        EditorDrawingTool::ThickLine => {
                            if let Some(edited_line_id) = editor.held_edited_thick_line {
                                let last = *editor.get_points_mut_of_thick_line(edited_line_id).last().unwrap();
                                editor.get_points_mut_of_thick_line(edited_line_id).push(last);
                            } else {
                                let working_color_id = match editor.selected_color { Some(x) => x, None => return };
                                let id = btreemap_usize_get_unused_id(&editor.scene.objects);
                                editor.scene.objects.insert(id, GroupedObjectNode {group: None,
                                    obj: ObjectNode::RealObjectNode(RealObjectNode {clone: None,
                                        att: RealObjectAttrs {
                                            shape: Shape::ThickLineShape(ThickLineShape{
                                                thickness: 2., points: vec![p, p], prolonged_tips: false
                                            }),
                                            color: working_color_id }
                                    })});
                                editor.held_edited_thick_line = Some(id);
                            }
                        }
                    }
                }
            })();
        }

        if is_mouse_button_released(MouseButton::Left) {
            editor.held_arrow_start_embroidery_control_group = None;
            editor.held_arrow_end_embroidery_control_group = None;
        }

        if let Some(group) = editor.held_arrow_start_embroidery_control_group {
            editor.scene.area_groups.get_mut(&group).unwrap().control_center_pos = editor.cam.screen_coord_to_scene(get_mouse_position_vec2());
        }

        if let Some(group_id) = editor.held_arrow_end_embroidery_control_group {
            let group: &mut AreaShapeGroup = editor.scene.area_groups.get_mut(&group_id).unwrap();
            let a = editor.cam.scene_coord_to_screen(group.control_center_pos);
            group.control_fill_dir_offset = (get_mouse_position_vec2() - a).normalize();
        }

        if let Some(mov_id) = editor.held_edited_sym {
            let x = editor.scene.movements.get_mut(&mov_id).unwrap();
            x.as_symmetry_movement_mut().unwrap().pos_y2 = editor.cam.get_mouse_scene_pos().y;
        }

        if let Some(area_id) = editor.held_edited_perimeter {
            let p = editor.cam.get_mouse_scene_pos();
            *editor.get_points_mut_of_area(area_id).last_mut().unwrap() = p;
        }

        if let Some(thin_line_id) = editor.held_edited_thin_line {
            let p = editor.cam.get_mouse_scene_pos();
            *editor.get_points_mut_of_thin_line(thin_line_id).last_mut().unwrap() = p;
        }

        if let Some(thick_line_id) = editor.held_edited_thick_line {
            let p = editor.cam.get_mouse_scene_pos();
            *editor.get_points_mut_of_thick_line(thick_line_id).last_mut().unwrap() = p;
        }

        if is_pressed_with_no_mod(KeyCode::Escape){
            if let Some(area_id) = editor.held_edited_perimeter {
                editor.delete_the_object(area_id, false);
            } else if let Some(id) = editor.held_edited_thin_line {
                editor.delete_the_object(id, false);
            } else if let Some(id) = editor.held_edited_thick_line {
                editor.delete_the_object(id, false);
            }
        }

        if is_pressed_with_no_mod(KeyCode::Z) {
            editor.finish_drawing_progress();
        }

        if is_pressed_with_no_mod(KeyCode::E){
            editor.stitches = None;
            editor.mode = EditorMode::Embroidery;
        }
        if is_pressed_with_no_mod(KeyCode::M){
            editor.stitches = None;
            editor.mode = EditorMode::Draw;
        }
        if is_pressed_with_shift(KeyCode::C) {
            // todo: do embroidery shit
        }
        if is_pressed_with_no_mod(KeyCode::Escape) && editor.mode.is_stitch() {
            editor.stitches = None;
            editor.mode = EditorMode::Embroidery;
        }
        if is_pressed_with_no_mod(KeyCode::Key1){
            editor.change_tool(EditorDrawingTool::Area);
        }
        if is_pressed_with_no_mod(KeyCode::Key2){
            editor.selected_tool = EditorDrawingTool::ThinLine;
        }
        if is_pressed_with_no_mod(KeyCode::Key3){
            editor.selected_tool = EditorDrawingTool::ThickLine;
        }
        if is_pressed_with_no_mod(KeyCode::Key4){
            editor.selected_tool = EditorDrawingTool::Symmetry;
        }

        if mouse_wheel().1 != 0. {
            let s = 1.15f32.powf(mouse_wheel().1);
            let pointing = editor.cam.screen_coord_to_scene(get_mouse_position_vec2());
            editor.cam.top_left = pointing + (-pointing + editor.cam.top_left) / s;
            editor.cam.px_in_mm = editor.cam.px_in_mm * s;
        }
        if is_no_mod_down() {
            let cam_movement_keys: &[KeyCode] = &[KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D];
            if t_started_moving.is_none() && cam_movement_keys.iter().any(|&k| {is_key_down(k)}) {
                t_started_moving = Some(t_now);
            } else if (t_started_moving.is_some() && cam_movement_keys.iter().all(|&k|{!is_key_down(k)})){
                t_started_moving = None;
            }
            if let Some(i_started_moving) = t_started_moving {
                let cam_pix_per_second = 800.0 + f32::min(0.5, (t_now - i_started_moving).as_secs_f32()) * 1000.;
                let delta = cam_pix_per_second / editor.cam.px_in_mm * (t_now - t_previous).as_secs_f32();

                if is_key_down(KeyCode::W){
                    editor.cam.top_left.y += delta;
                }
                if is_key_down(KeyCode::A){
                    editor.cam.top_left.x -= delta;
                }
                if is_key_down(KeyCode::S){
                    editor.cam.top_left.y -= delta;
                }
                if is_key_down(KeyCode::D){
                    editor.cam.top_left.x += delta;
                }
            }
        }
        if is_quit_requested(){
            // todo: save
            break;
        }
        next_frame().await;
        t_previous = t_now;
    }
}

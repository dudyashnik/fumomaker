#![feature(int_roundings)]

mod scene;
mod editor;
mod embroidery;
mod util;

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

fn get_mouse_position_vec2() -> Vec2 {
    vec2(mouse_position().0, mouse_position().1)
}

fn get_different_gray_color(c: Vec3) -> Vec3 {
    let intensity = 0.21 * c.x + 0.71 * c.y + 0.08 * c.z;
    let d = (|x| { if (x < 0.5) {1.0} else {0.0} })(intensity);
    vec3(d, d, d)
}

fn vec3_to_mq_color(c: Vec3) -> Color {
    Color::new(c.x, c.y, c.z, 1.0)
}

#[derive(Clone)]
struct CoolColor{
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

#[derive(Clone)]
struct Camera {
    px_in_mm: f32,
    top_left: Vec2,
}

impl Camera {
    fn scene_coord_to_screen(&self, scene_c: Vec2) -> Vec2{
        let n = scene_c - self.top_left;
        vec2(n.x * self.px_in_mm, -n.y * self.px_in_mm)
    }

    fn screen_coord_to_scene(&self, screen_c: Vec2) -> Vec2{
        self.top_left + vec2(screen_c.x / self.px_in_mm, -screen_c.y / self.px_in_mm)
    }

    fn draw_mq_line_on_scene(&self, a: Vec2, b: Vec2, thickness: f32, color: Color) {
        let sa = self.scene_coord_to_screen(a);
        let sb = self.scene_coord_to_screen(b);
        draw_line(sa.x, sa.y, sb.x, sb.y, thickness, color)
    }

    fn draw_arrow(&self, a: Vec2, b: Vec2, thickness: f32, color: Color){
        let sa = self.scene_coord_to_screen(a);
        let sb = self.scene_coord_to_screen(b);
        let back = (-sb + sa).normalize() * 5.;
        let p1 = sb + get_rot_mat(PI * 0.25) * back;
        let p2 = sb + get_rot_mat(-PI * 0.25) * back;
        draw_line(sa.x, sa.y, sb.x, sb.y, thickness, color);
        draw_line(p1.x, p1.y, sb.x, sb.y, thickness, color);
        draw_line(p2.x, p2.y, sb.x, sb.y, thickness, color);
    }

    fn draw_geom_segment(&self, a: Vec2, b: Vec2, thickness: f32, break_thickness : f32, color: Color) {
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

    fn draw_smol_dot(&self, a: Vec2, rad: f32, color: Color){
        let s = self.screen_coord_to_scene(a);
        draw_circle(s.x, s.y, rad, color);
    }

    fn draw_smol_ring(&self, a: Vec2, rad: f32, thickness: f32, color: Color){
        let s = self.screen_coord_to_scene(a);
        draw_circle_lines(s.x, s.y, rad, thickness, color);
    }

    fn draw_smol_circle(&self, a: Vec2, rad: f32, inner_color: Color, thickness: f32, perimeter_color: Color) {
        let s = self.screen_coord_to_scene(a);
        draw_circle(s.x, s.y, rad, inner_color);
        draw_circle_lines(s.x, s.y, rad + thickness, thickness, perimeter_color);
    }

    fn draw_smol_square(&self, a: Vec2, rad: f32, inner_color: Color, thickness: f32, perimeter_color: Color){
        let s = self.screen_coord_to_scene(a);
        draw_rectangle(s.x - rad, s.y - rad, rad * 2., rad * 2., inner_color);
        draw_rectangle_lines(s.x - rad - thickness, s.y - rad - thickness,
                             (rad + thickness) * 2., (rad + thickness) * 2., thickness, perimeter_color);
    }

    fn draw_strip_line(&self, scene_a: Vec2, scene_b: Vec2, px_offset: f32, thickness: f32,
                       worm_length: f32, worm_tail_dist: f32, color: Color){
        let sa = self.scene_coord_to_screen(scene_a);
        let sb = self.scene_coord_to_screen(scene_b);
        let l_full = (-sa+sb).length();
        let mut a = ((sa.x as f64 * 256.0).round() as i64, (sa.y as f64 * 256.0).round() as i64);
        let mut b = ((sa.x as f64 * 256.0).round() as i64, (sa.y as f64 * 256.0).round() as i64);
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
}


impl From<&UsedColor> for Color {
    fn from(c :&UsedColor) -> Self {
        Self::new(c.clr.x, c.clr.y, c.clr.z, 1f32)
    }
}

impl From<&CoolColor> for UsedColor {
    fn from(c: &CoolColor) -> Self {
        Self {clr: c.clr, name: String::from(c.name)}
    }
}

impl Scene {
    fn get_bg_color(&self) -> Vec3 {
        vec3(1., 1., 1.)
    }
}

#[derive(EnumAsInner)]
enum EditorMode {
    // In which we draw shapes
    Draw,
    // In which we select direction for area groups and embroidery params
    Embroidery,
    // In which we see the result
    Stitch,
}

enum EditorDrawingTool{
    Area,
    ThinLine,
    ThickLine,
    Symmetry,
}

struct Editor {
    scene: Scene,
    cam: Camera,
    mode: EditorMode,
    // `stitches` will be shown only in Stitch mode
    stitches: Option<EmbroideryImage>,
    selected_color: Option<usize>,
    selected_tool: EditorDrawingTool,
    selected_shape: Option<usize>,
    selected_movement: Option<usize>,
    selected_point: Option<usize>,
    selected_point_my_start: Vec2,
    selected_point_origin_start: Vec2,
    unsaved: bool,
}

impl Editor{
    fn new(scene: Scene) -> Editor {
        Editor {scene, cam: Camera {top_left: Vec2::new(-50.0, 50.0), px_in_mm: 33.0},
            mode: EditorMode::Draw,
            stitches: None,
            selected_color: None,
            selected_tool: EditorDrawingTool::Area,
            selected_shape: None,
            selected_movement: None,
            selected_point: None,
            selected_point_my_start: Vec2::default(),
            selected_point_origin_start: Vec2::default(),
            unsaved: false,
        }
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

    let draw_my_font = |txt: &str, x: f32, y: f32, color: Color, font_size: u16| {
        draw_text_ex(txt, x, y, TextParams {color, font: Some(&font),
            font_size, font_scale: 1f32, ..Default::default()});
    };

    let draw_rectangle_with_borders = |px: f32, py: f32, width: f32, height: f32, border_clr: Color, clr: Color|{
        let ob = 1.0;
        draw_rectangle_lines(px - ob, py - ob, width + ob * 2f32, height + ob * 2f32, ob * 2f32, border_clr);
        draw_rectangle(px, py, width, height, clr);
    };

    let draw_color_descr_square = |px: f32, py: f32, dim: f32, color: Vec3|{
        let ob = 1.0;
        draw_rectangle_lines(px - ob, py - ob, dim + ob * 2f32, dim + ob * 2f32,
                             ob * 2f32, vec3_to_mq_color(get_different_gray_color(color)));
        draw_rectangle(px, py, dim, dim, Color::new(color.x, color.y, color.z, 1.0));
    };

    let mut editor = Editor::new(Scene::default());

    let draw_grid = |editor: &Editor|{
        let scene: &Scene = &editor.scene;
        let draw_vertical = |thickness: f32, color: Color, text_color: Option<Color>, x: f32, magnitude: i32|{
            let screen_x = editor.cam.scene_coord_to_screen(vec2(x, 0.0)).x;
            draw_line(screen_x, if text_color.is_some() {0.} else {20.}, screen_x, screen_height(), thickness, color);
            if let Some(text_color) = text_color {
                draw_my_font(&format_float(x, -magnitude), screen_x + thickness / 2. + 2.0, 17.0, text_color, 14);
            }
        };
        let draw_horizontal = |thickness: f32, color: Color, text_color: Option<Color>, y: f32, magnitude: i32|{
            let screen_y = editor.cam.scene_coord_to_screen(vec2(0.0, y)).y;
            draw_line(if text_color.is_some() {0.} else {40.}, screen_y, screen_width(), screen_y, thickness, color);
            if let Some(text_color) = text_color {
                draw_my_font(&format_float(y, -magnitude), 2.0, screen_y + thickness / 2. + 17., text_color, 14);
            }
        };
        let choose_color = |alpha: f32| -> Color {
            let bg = scene.get_bg_color();
            vec3_to_mq_color(bg * (1. - alpha) + get_different_gray_color(bg) * alpha)
            // LIGHTGRAY
        };
        let draw_lines = |alpha: f32, thickness: f32, magnitude: i32, gap: f32, skip_dec: bool, draw_text: bool|{
            let color = choose_color(alpha);
            let text_color = choose_color(0.95);
            let left = editor.cam.top_left.x;
            let right = editor.cam.top_left.x + screen_width() / editor.cam.px_in_mm;
            for i in ((left / gap).ceil() as i64)..((right / gap).ceil() as i64) {
                if (i % 10 == 0 && skip_dec) || i == 0 {
                    continue
                }
                let x = (i as f32) * gap;
                draw_vertical(thickness, color, Some(text_color).filter(|_| draw_text), x, magnitude);
            }
            let bottom = editor.cam.top_left.y - screen_height() / editor.cam.px_in_mm;
            let top = editor.cam.top_left.y;
            for i in ((bottom / gap).ceil() as i64)..((top / gap).ceil() as i64) {
                if (i % 10 == 0 && skip_dec) || i == 0 {
                    continue;
                }
                let y = i as f32 * gap;
                draw_horizontal(thickness, color, Some(text_color).filter(|_| draw_text), y, magnitude);
            }
        };
        let min_visibility = 9.0;
        let min_real = min_visibility * 10.;
        for magnitude in -2..3 {
            let gap = 10f32.powi(magnitude);
            let px_gap = editor.cam.px_in_mm * gap;
            if px_gap > min_visibility {
                let ascent_1 = f32::clamp((px_gap - min_visibility) / (min_real - min_visibility), 0., 1.);
                draw_lines(0.   + ascent_1 * 0.15, 0. + ascent_1 * 2., magnitude, gap, true, false);
                draw_lines(0.15 + ascent_1 * 0.25, 2. + ascent_1 * 1., magnitude + 1, gap * 10., true, true);
                draw_lines(0.5  + ascent_1 * 0.2,  3. + ascent_1 * 1., magnitude + 2, gap * 100., false, true);
                break
            }
        }
        draw_vertical(5., choose_color(0.8), Some(choose_color(1.)), 0., 0);
        draw_horizontal(5., choose_color(0.8), Some(choose_color(1.)), 0., 0);
    };

    let draw_scene = |editor: &Editor|{
        let scene: &Scene = &editor.scene;
        for (&id, mov) in &scene.movements {
            match mov {
                MovementNode::SymmetryMovement(sym) => {
                    let color: Color = if let Some(s_mov_id) = editor.selected_movement && s_mov_id == id { ORANGE } else { GRAY };
                    editor.cam.draw_geom_segment(vec2(sym.x, sym.pos_y1), vec2(sym.x, sym.pos_y2), 12., 6., color);
                }
            }
        }

        for (&id, g_object) in &scene.objects {
            let object = &g_object.obj;
            let (source, trans): (&RealObjectAttrs, Option<MovementNode>) = match &g_object.obj {
                ObjectNode::RealObjectNode(real) => (&real.att, None),
                ObjectNode::GhostObject(ghost) => (
                    &scene.objects[&ghost.source].obj.as_real_object_node().unwrap().att,
                    Some(scene.movements[&ghost.movement])
                )
            };
            let color = vec3_to_mq_color(scene.colors[&source.color].clr);
            match &source.shape {
                Shape::AreaShape(_) => { continue }
                Shape::LineShape(line) => {
                    for i in 0..(line.points.len() - 1){
                        editor.cam.draw_mq_line_on_scene(line.points[i], line.points[i + 1], 2.,  color);
                    }
                },
                Shape::ThickLineShape(line) => {
                    assert!(line.points.len() >= 2);
                    let px_thickness = f32::max(2., editor.cam.px_in_mm * line.thickness);
                    for i in 1..(line.points.len() - 1){
                        editor.cam.draw_smol_dot(line.points[i], px_thickness,  color);
                    }
                    if line.prolonged_tips{
                        editor.cam.draw_smol_dot(line.points[0], px_thickness,  color);
                        editor.cam.draw_smol_dot(line.points[line.points.len() - 1], px_thickness,  color);
                    }
                    for i in 0..(line.points.len()) {
                        editor.cam.draw_mq_line_on_scene(line.points[i], line.points[i + 1], px_thickness, color);
                    }
                }
            }
        }
        for (&gid, group) in &scene.area_groups {
            let color = scene.colors[&scene.get_object_attrs(&scene.objects[&group.perimeters.first().unwrap()].obj).color].clr;
            if editor.mode.is_draw() || editor.mode.is_stitch() {
                for obj in group.perimeters.iter().map(|pid| -> &ObjectNode {&scene.objects[pid].obj} ) {
                    let (source, trans): (&AreaShape, Option<MovementNode>) = match obj {
                        ObjectNode::RealObjectNode(real) => (real.att.shape.as_area_shape().unwrap(), None),
                        ObjectNode::GhostObject(ghost) => (
                            scene.objects[&ghost.source].obj.as_real_object_node().unwrap().att.shape.as_area_shape().unwrap(),
                            Some(scene.movements[&ghost.movement])
                        )
                    };
                    for i in 0..source.points.len() {
                        let ni = if i + 1 == source.points.len() { 0 } else {i + 1};
                        let a = MovementNode::option_forward(trans, source.points[i]);
                        let b = MovementNode::option_forward(trans, source.points[ni]);
                        editor.cam.draw_mq_line_on_scene(a, b, 4., vec3_to_mq_color(color));
                    }
                }
            }
            if editor.mode.is_embroidery() {
                let (hidden, primary) = get_two_trench_zones(
                    &editor.scene, &group.perimeters, group.control_center_pos, group.control_fill_dir_offset, group.f_params);
                for (trench, thickness, alpha) in [(hidden, 3., 0.35), (primary, 5., 0.67)] {
                    for (yi, line) in trench.lines.iter().enumerate() {
                        for seg in line {
                            let a = trench.b + trench.a * vec2(seg.start, yi as f32 * trench.dist);
                            let b = trench.b + trench.a * vec2(seg.end, yi as f32 * trench.dist);
                            editor.cam.draw_mq_line_on_scene(a, b, thickness, Color::new(0., 0., 0., alpha));
                        }
                    }
                }
            }
        }

        if editor.mode.is_stitch() {
            let compiled: &EmbroideryImage = editor.stitches.as_ref().unwrap();
            for color_grp in &compiled.grp {
                let color = &editor.scene.colors[&color_grp.color];
                for path in &color_grp.paths {
                    editor.cam.draw_smol_circle(path.start, 7., vec3_to_mq_color(color.clr), 2., BLUE);
                    let mut prev = path.start;
                    for stitch in &path.stitches {
                        editor.cam.draw_arrow(prev, stitch.end, 2., match stitch.kind {
                            StitchKind::Normal => BLUE,
                            StitchKind::NormalBorder => Color::new(0.7, 0.7, 1., 1.),
                            StitchKind::HoppingInDescend => RED,
                            StitchKind::HoppingToTurnBack => Color::new(0.8, 0.2, 0.8, 1.),
                            StitchKind::HoppingStartOfJob => Color::new(1., 0.7, 0.7, 1.),
                            StitchKind::TransLevel => GREEN,
                        });
                        prev = stitch.end;
                    }
                }
            }
        }
    };

    let draw_info_label = |editor: &Editor|{
        let editor_status = match editor.mode {
            EditorMode::Draw => format!("Drawing mode. {}", match editor.selected_tool {
                EditorDrawingTool::Symmetry => "Symmetry line tool",
                EditorDrawingTool::Area => "Closed area drawing tool",
                EditorDrawingTool::ThinLine => "Thin path drawing tool",
                EditorDrawingTool::ThickLine => "Thick path drawing tool",
            }),
            EditorMode::Embroidery => format!("Configuring embroidery"),
            EditorMode::Stitch => format!("Embroidery finished. Will export to {}", embroidery_file_name),
        };
        let info_text = format!("{}{}", if editor.unsaved { "*Unsaved* "} else {""}, editor_status);

        let dim = measure_text(&info_text, Some(&font), 30, 1f32);
        let margin = 8f32;
        let top_padding = 7.0;
        draw_rectangle_with_borders((screen_width() - dim.width) / 2f32 - margin, top_padding, dim.width + margin * 2.0, dim.height + margin * 2.0, LIGHTGRAY, WHITE);
        draw_my_font(&info_text, (screen_width() - dim.width) / 2f32, top_padding + margin + dim.offset_y, BLACK, 30);
    };

    let draw_color_list = |editor: &Editor|{
        let d = 30f32;
        let padding_right = 10f32;
        let margin = 6f32;
        let text_margin_square = 15f32;
        let font_sz = 22;
        let mut y = 10f32;
        for (&id, clr) in &editor.scene.colors {
            let label = format!("{} (#{})", clr.name, id);
            let text_dim = measure_text(&label, Some(&font), font_sz, 1.0);
            let fw = margin + text_dim.width + text_margin_square + d + margin;
            if let Some(sc_id) = editor.selected_color && sc_id == id {
                draw_rectangle_lines(screen_width() - fw - padding_right, y, fw, margin + d + margin, 4.0, ORANGE);
            }
            draw_my_font(&label, screen_width() - fw - padding_right + margin, y + margin + d - 10.0, BLACK, font_sz);
            draw_color_descr_square(screen_width() - padding_right - margin - d, y + margin,
                                        d, clr.clr);
            y += margin + d + margin;
        }
    };

    let draw_node_list = |editor: &Editor|{
        let scene: &Scene = &editor.scene;
        let ic_h = 30.0;
        let margin = 5.0;
        let square_margin_label = 7.0;
        let padding_bot = 3.0;
        let padding_left_normal = 10f32;
        let padding_left_tabbed = 45f32;
        let sum_height: f32 = 10.0 +
            (scene.area_groups.len() +
            scene.objects.len() + scene.movements.len()) as f32 *
                (padding_bot + margin * 2.0 + ic_h);
        let mut y = screen_height() - sum_height;
        for (&id, mov) in &scene.movements {
            draw_rectangle(padding_left_normal, y + margin, ic_h, ic_h, GRAY);
            let label = format!("Symmetry line #{}", id);
            draw_my_font(&label, padding_left_normal + ic_h + square_margin_label, y + margin + ic_h, BLACK, 23);
            y += margin * 2.0 + ic_h + padding_bot;
        }
        let write_object_label = |my_id: usize, obj: &GroupedObjectNode| {
            format!("{} #{}{}", match scene.get_object_attrs(&obj.obj).shape {
                Shape::LineShape(_) => "Thin path",
                Shape::ThickLineShape(_) => "Thick path",
                Shape::AreaShape(_) => "Closed area"
            }, my_id, match &obj.obj {
                ObjectNode::RealObjectNode(_) => "".to_string(),
                ObjectNode::GhostObject(gh) => format!(", ghost of #{} through #{}", gh.source, gh.movement),
            })
        };
        for (&id, obj) in &scene.objects {
            if let Shape::AreaShape(_) = scene.get_object_attrs(&obj.obj).shape {
                continue;
            }
            let color = scene.colors[&scene.get_object_attrs(&obj.obj).color].clr;
            draw_color_descr_square(padding_left_normal, y + margin, ic_h, color);
            draw_my_font(&write_object_label(id, obj),
                         padding_left_normal + ic_h + square_margin_label, y + margin + ic_h, BLACK, 23);
            y += margin * 2.0 + ic_h + padding_bot;
        }
        for (&group_id, group) in &scene.area_groups {
            let group_color_id = scene.get_object_attrs(&scene.objects[group.perimeters.first().unwrap()].obj).color;
            let group_color = scene.colors[&group_color_id].clr;
            draw_color_descr_square(padding_left_normal, y + margin, ic_h, group_color);
            draw_my_font(&format!("Area group #{}", group_id),
                         padding_left_normal + ic_h + square_margin_label, y + margin + ic_h, BLACK, 23);
            y += margin * 2.0 + ic_h + padding_bot;

            for &area_obj_id in &group.perimeters{
                let obj = &scene.objects[&area_obj_id];
                draw_color_descr_square(padding_left_tabbed, y + margin, ic_h, group_color);
                draw_my_font(&write_object_label(area_obj_id, obj),
                         padding_left_tabbed + ic_h + square_margin_label, y + margin + ic_h, BLACK, 23);
                y += margin * 2.0 + ic_h + padding_bot;
            }
        }
    };

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
        draw_grid(&editor);
        draw_scene(&editor);
        draw_info_label(&editor);
        draw_color_list(&editor);
        draw_node_list(&editor);

        if is_key_pressed(KeyCode::E){
            editor.stitches = None;
            editor.mode = EditorMode::Embroidery;
        }
        if is_key_pressed(KeyCode::M){
            editor.stitches = None;
            editor.mode = EditorMode::Draw;
        }

        if mouse_wheel().1 != 0. {
            let s = 1.15f32.powf(mouse_wheel().1);
            let pointing = editor.cam.screen_coord_to_scene(get_mouse_position_vec2());
            editor.cam.top_left = pointing + (-pointing + editor.cam.top_left) / s;
            editor.cam.px_in_mm = editor.cam.px_in_mm * s;
        }
        {
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

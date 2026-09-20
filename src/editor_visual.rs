use enum_as_inner::EnumAsInner;

use macroquad::prelude::glam::*;
use macroquad::color::*;
use macroquad::shapes::*;
use macroquad::window::*;
use macroquad::text::*;

use std::time::*;
use std::f32::consts::PI;

use crate::embroidery::*;
use crate::util::*;
use crate::drawing::*;

impl Scene {
    pub fn get_bg_color(&self) -> Vec3 {
        COLOR_VEC_WHITE
    }
}

#[derive(EnumAsInner, Copy, Clone, PartialEq)]
pub enum EditorMode {
    // In which we draw shapes
    Draw,
    // In which we select direction for area groups and embroidery params
    Embroidery,
    // In which we see the result
    Stitch,
}

#[derive(EnumAsInner, Copy, Clone, Eq, PartialEq)]
pub enum EditorDrawingTool{
    Area,
    ThinLine,
    ThickLine,
    Symmetry,
}

#[derive(EnumAsInner, Copy, Clone, Eq, PartialEq)]
pub enum Selection {
    Nothing,
    Object(usize),
    Movement(usize),
}

pub struct Editor {
    pub scene: Scene,
    pub cam: Camera,
    pub mode: EditorMode,
    // `stitches` will be shown only in Stitch mode
    pub stitches: Option<EmbroideryImage>,
    pub selected_color: Option<usize>,
    pub selected_tool: EditorDrawingTool,
    pub selected: Selection,
    pub selected_shape_for_attaching_to_group: Option<usize>,
    pub selected_shape_for_moving: Option<usize>,
    pub selected_movement_for_movement: Option<usize>,
    pub selected_point: Option<usize>,
    pub selected_point_my_start: Vec2,
    pub selected_point_origin_start: Vec2,
    pub unsaved: bool,
    pub held_arrow_start_embroidery_control_group: Option<usize>,
    pub held_arrow_end_embroidery_control_group: Option<usize>,
    pub held_edited_sym: Option<usize>,
    pub held_edited_thin_line: Option<usize>,
    pub held_edited_thick_line: Option<usize>,
    pub held_edited_perimeter: Option<usize>,
    pub t_started_moving: Option<Instant>,
}

impl Editor {
    pub fn new(scene: Scene) -> Editor {
        Editor {
            scene,
            cam: Camera { top_left: Vec2::new(-50.0, 50.0), px_in_mm: 33.0 },
            mode: EditorMode::Draw,
            stitches: None,
            selected_color: None,
            selected_tool: EditorDrawingTool::Area,
            selected: Selection::Nothing,
            selected_shape_for_attaching_to_group: None,
            selected_shape_for_moving: None,
            selected_movement_for_movement: None,
            selected_point: None,
            selected_point_my_start: Vec2::default(),
            selected_point_origin_start: Vec2::default(),
            unsaved: false,
            held_arrow_start_embroidery_control_group: None,
            held_arrow_end_embroidery_control_group: None,
            held_edited_sym: None,
            held_edited_thin_line: None,
            held_edited_thick_line: None,
            held_edited_perimeter: None,
            t_started_moving: None,
        }
    }

    pub fn draw_grid(&self, font: &Font) {
        let scene: &Scene = &self.scene;
        let draw_vertical = |thickness: f32, color: Color, text_color: Option<Color>, x: f32, magnitude: i32| {
            let screen_x = self.cam.scene_coord_to_screen(vec2(x, 0.0)).x;
            draw_line(screen_x, if text_color.is_some() { 0. } else { 20. }, screen_x, screen_height(), thickness, color);
            if let Some(text_color) = text_color {
                draw_my_font(font, &format_float(x, -magnitude), screen_x + thickness / 2. + 2.0, 17.0, text_color, 14);
            }
        };
        let draw_horizontal = |thickness: f32, color: Color, text_color: Option<Color>, y: f32, magnitude: i32| {
            let screen_y = self.cam.scene_coord_to_screen(vec2(0.0, y)).y;
            draw_line(if text_color.is_some() { 0. } else { 40. }, screen_y, screen_width(), screen_y, thickness, color);
            if let Some(text_color) = text_color {
                draw_my_font(font, &format_float(y, -magnitude), 2.0, screen_y + thickness / 2. + 17., text_color, 14);
            }
        };
        let choose_color = |alpha: f32| -> Color {
            let bg = scene.get_bg_color();
            vec3_to_mq_color(bg * (1. - alpha) + get_different_gray_color(bg) * alpha)
            // LIGHTGRAY
        };
        let draw_lines = |alpha: f32, thickness: f32, magnitude: i32, gap: f32, skip_dec: bool, draw_text: bool| {
            let color = choose_color(alpha);
            let text_color = choose_color(0.95);
            let left = self.cam.top_left.x;
            let right = self.cam.top_left.x + screen_width() / self.cam.px_in_mm;
            for i in ((left / gap).ceil() as i64)..((right / gap).ceil() as i64) {
                if (i % 10 == 0 && skip_dec) || i == 0 {
                    continue
                }
                let x = (i as f32) * gap;
                draw_vertical(thickness, color, Some(text_color).filter(|_| draw_text), x, magnitude);
            }
            let bottom = self.cam.top_left.y - screen_height() / self.cam.px_in_mm;
            let top = self.cam.top_left.y;
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
            let px_gap = self.cam.px_in_mm * gap;
            if px_gap > min_visibility {
                let ascent_1 = f32::clamp((px_gap - min_visibility) / (min_real - min_visibility), 0., 1.);
                draw_lines(0. + ascent_1 * 0.15, 0. + ascent_1 * 2., magnitude, gap, true, false);
                draw_lines(0.15 + ascent_1 * 0.25, 2. + ascent_1 * 1., magnitude + 1, gap * 10., true, true);
                draw_lines(0.5 + ascent_1 * 0.2, 3. + ascent_1 * 1., magnitude + 2, gap * 100., false, true);
                break
            }
        }
        draw_vertical(5., choose_color(0.8), Some(choose_color(1.)), 0., 0);
        draw_horizontal(5., choose_color(0.8), Some(choose_color(1.)), 0., 0);
    }

    // Returns screen coords
    pub fn get_area_group_control_arrow_scr_pos(&self, group: &AreaShapeGroup) -> (Vec2, Vec2) {
        let arr_a = self.cam.scene_coord_to_screen(group.control_center_pos);
        let arr_b = arr_a + group.control_fill_dir_offset.normalize() * 250.;
        (arr_a, arr_b)
    }

    pub fn draw_scene(&self, t: f32) {
        let scene: &Scene = &self.scene;
        for (&id, mov) in &scene.movements {
            match mov {
                MovementNode::SymmetryMovement(sym) => {
                    let (a, b) = (vec2(sym.x, sym.pos_y1), vec2(sym.x, sym.pos_y2));
                    self.cam.draw_geom_segment(a, b, 12., 6.,
                        if self.selected == Selection::Movement(id) { ORANGE } else { GRAY });
                    if self.selected == Selection::Movement(id) {
                        self.cam.draw_dash_line(a, b, 5.0 * t, 16., 20., 40., vec3_to_mq_color(COLOR_VEC_ORANGE))
                    } else if self.selected_movement_for_movement == Some(id) {
                        self.cam.draw_dash_line(a, b, 5.0 * t, 16., 20., 40., vec3_to_mq_color(COLOR_VEC_BLUE))
                    }
                }
            }
        }

        for (&id, g_object) in &scene.objects {
            let (source, trans) = scene.get_source_and_transition_of_object_node(&g_object.obj);
            let color = vec3_to_mq_color(scene.colors[&source.color].clr);
            let (scene_points, thickness) = match &source.shape {
                Shape::AreaShape(_) => { continue }
                Shape::LineShape(line) => {
                    for i in 0..(line.points.len() - 1) {
                        self.cam.draw_mq_line_on_scene(line.points[i], line.points[i + 1], 2., color);
                    }
                    (&line.points, 2.)
                },
                Shape::ThickLineShape(line) => {
                    assert!(line.points.len() >= 2);
                    let px_thickness = f32::max(2., self.cam.px_in_mm * line.thickness);
                    for i in 1..(line.points.len() - 1) {
                        self.cam.draw_circle(line.points[i], px_thickness / 2., color);
                    }
                    if line.prolonged_tips {
                        self.cam.draw_circle(line.points[0], px_thickness, color);
                        self.cam.draw_circle(line.points[line.points.len() - 1], px_thickness / 2., color);
                    }
                    (&line.points, px_thickness)
                }
            };
            let points: Vec<Vec2> = scene_points.iter().map(|scr|{self.cam.scene_coord_to_screen(MovementNode::option_forward(trans, *scr))}).collect();
            Camera::draw_path_on_screen(&points, thickness, color);
            if self.selected == Selection::Object(id){
                Camera::draw_dashed_path_on_screen(&points, 5. * t, thickness + 2., 20., 40., vec3_to_mq_color(COLOR_VEC_ORANGE));
            } else if self.selected_shape_for_attaching_to_group == Some(id){
                Camera::draw_dashed_path_on_screen(&points, 5. * t, thickness + 2., 20., 40., vec3_to_mq_color(COLOR_VEC_BLUE_2));
            } else if self.selected_shape_for_moving == Some(id) {
                Camera::draw_dashed_path_on_screen(&points, 5. * t, thickness + 2., 20., 40., vec3_to_mq_color(COLOR_VEC_BLUE));
            }
        }
        for (&gid, group) in &scene.area_groups {
            let color = scene.colors[&scene.get_object_attrs_by_id(*group.perimeters.first().unwrap()).color].clr;
            if self.mode.is_draw() {
                for (area_id, obj) in group.perimeters.iter().map(|&id| -> (usize, &ObjectNode) { (id, &scene.objects[&id].obj) }) {
                    let (source, trans): (&AreaShape, Option<MovementNode>) = {
                        let (source_attrs, trans) = scene.get_source_and_transition_of_object_node(obj);
                        (source_attrs.shape.as_area_shape().unwrap(), trans)
                    };
                    let points: Vec<Vec2> = source.points.iter().map(|scr|{self.cam.scene_coord_to_screen(MovementNode::option_forward(trans, *scr))}).collect();
                    if source.is_gap {
                        Camera::draw_dashed_contour_on_screen(&points, 1. * t, 4., 28., 30.,
                                    vec3_to_mq_color(color), self.held_edited_perimeter != Some(area_id));
                    } else {
                        Camera::draw_contour_on_screen(&points, 4., vec3_to_mq_color(color),
                                    self.held_edited_perimeter != Some(area_id));
                    }
                    if self.selected == Selection::Object(area_id){
                        Camera::draw_dashed_contour_on_screen(&points, 5. * t, 6., 20., 40., vec3_to_mq_color(COLOR_VEC_ORANGE), true);
                    } else if self.selected_shape_for_attaching_to_group == Some(area_id){
                        Camera::draw_dashed_contour_on_screen(&points, 5. * t, 6., 20., 40., vec3_to_mq_color(COLOR_VEC_BLUE_2), true);
                    } else if self.selected_shape_for_moving == Some(area_id) {
                        Camera::draw_dashed_contour_on_screen(&points, 5. * t, 6., 20., 40., vec3_to_mq_color(COLOR_VEC_BLUE), true);
                    }
                }
            }
            if self.mode.is_embroidery() {
                let true_dir = vec2(group.control_fill_dir_offset.x, -group.control_fill_dir_offset.y);
                let (hidden, primary) = get_two_trench_zones(
                    &self.scene, &group.perimeters, group.control_center_pos, true_dir, group.f_params);
                for (trench, thickness, alpha) in [(hidden, 3., 0.35), (primary, 5., 0.67)] {
                    for (yi, line) in trench.lines.iter().enumerate() {
                        for seg in line {
                            let a = trench.b + trench.a * vec2(seg.start, yi as f32 * trench.dist);
                            let b = trench.b + trench.a * vec2(seg.end, yi as f32 * trench.dist);
                            self.cam.draw_mq_line_on_scene(a, b, thickness, Color::new(0., 0., 0., alpha));
                        }
                    }
                }

                let dir_arrow_color = vec3_to_mq_color(vec3(0.3, 0., 0.));
                let (arr_a, arr_b) = self.get_area_group_control_arrow_scr_pos(group);
                Camera::draw_arrow_on_screen(arr_a, arr_b, 4., dir_arrow_color);
                Camera::draw_decor_square_on_screen(arr_a, 3., WHITE, 3., dir_arrow_color);
                Camera::draw_circle_with_perimeter_on_screen(arr_b, 4., BLANK, 2., dir_arrow_color);
            }
        }

        if self.mode.is_stitch() {
            let compiled: &EmbroideryImage = self.stitches.as_ref().unwrap();
            for color_grp in &compiled.grp {
                let color = &color_grp.color;
                for path in &color_grp.paths {
                    self.cam.draw_circle_with_perimeter(path.start, 7., vec3_to_mq_color(color.clr), 2., BLUE);
                    let mut prev = path.start;
                    for stitch in &path.stitches {
                        self.cam.draw_arrow(prev, stitch.end, 2., match stitch.kind {
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
    }

    pub fn draw_info_label(&self, font: &Font, embroidery_file_name: &str) {
        let editor_status = match self.mode {
            EditorMode::Draw => format!("Drawing mode. {}", match self.selected_tool {
                EditorDrawingTool::Symmetry => "Symmetry line tool",
                EditorDrawingTool::Area => "Closed area drawing tool",
                EditorDrawingTool::ThinLine => "Thin path drawing tool",
                EditorDrawingTool::ThickLine => "Thick path drawing tool",
            }),
            EditorMode::Embroidery => format!("Configuring embroidery"),
            EditorMode::Stitch => format!("Embroidery finished. Will export to {}", embroidery_file_name),
        };
        let info_text = format!("{}{}", if self.unsaved { "*Unsaved* " } else { "" }, editor_status);

        let dim = measure_text(&info_text, Some(font), 30, 1f32);
        let margin = 8f32;
        let top_padding = 7.0;
        draw_rectangle_with_borders((screen_width() - dim.width) / 2f32 - margin, top_padding, dim.width + margin * 2.0, dim.height + margin * 2.0, LIGHTGRAY, WHITE);
        draw_my_font(font, &info_text, (screen_width() - dim.width) / 2f32, top_padding + margin + dim.offset_y, BLACK, 30);
    }

    pub fn draw_color_list(&self, font: &Font) {
        let d = 30f32;
        let padding_right = 10f32;
        let margin = 6f32;
        let text_margin_square = 15f32;
        let font_sz = 22;
        let mut y = 10f32;
        for (&id, clr) in &self.scene.colors {
            let label = format!("{} (#{})", clr.name, id);
            let text_dim = measure_text(&label, Some(font), font_sz, 1.0);
            let fw = margin + text_dim.width + text_margin_square + d + margin;
            if let Some(sc_id) = self.selected_color && sc_id == id {
                draw_rectangle_lines(screen_width() - fw - padding_right, y, fw, margin + d + margin, 4.0, ORANGE);
            }
            draw_my_font(font, &label, screen_width() - fw - padding_right + margin, y + margin + d - 10.0, BLACK, font_sz);
            draw_color_descr_square(screen_width() - padding_right - margin - d, y + margin,
                                    d, clr.clr);
            y += margin + d + margin;
        }
    }
}

struct EntryInSideList {
    border_color: Vec3,
    pin_color: Vec3,
    text: String,
    tab: i32,
}

impl Editor {
    fn draw_list_of_nodes(&self, entries: &[EntryInSideList], font: &Font){
        let ic_h = 30.0;
        let margin = 5.0;
        let square_margin_label = 7.0;
        let padding_bot = 3.0;
        let padding_left_normal = 10f32;
        let padding_left_tabbed = 35f32;
        let sum_height: f32 = 10.0 + entries.len() as f32 * (padding_bot + margin * 2.0 + ic_h);
        let font_sz = 23;

        let mut y = screen_height() - sum_height;
        for e in entries {
            let pad = padding_left_normal + padding_left_tabbed * e.tab as f32;
            let text_dim = measure_text(&e.text, Some(font), font_sz, 1f32);
            draw_rectangle_with_borders(pad, y,
                                        margin * 3. + ic_h + square_margin_label + text_dim.width, margin * 2.0 + ic_h,
                                        vec3_to_mq_color(e.border_color), WHITE);
            draw_color_descr_square(pad + margin, y + margin, ic_h, e.pin_color);
            draw_my_font(font, &e.text,
                         pad + margin + ic_h + square_margin_label,
                         y + margin + ic_h, BLACK, font_sz);
            y += margin * 2.0 + ic_h + padding_bot;
        }
    }

    pub fn draw_node_list(&self, font: &Font) {
        let scene: &Scene = &self.scene;
        let mut entries: Vec<EntryInSideList> = Vec::with_capacity(scene.area_groups.len() +
            scene.objects.len() + scene.movements.len());

        for (&id, mov) in &scene.movements {
            entries.push(EntryInSideList{border_color: COLOR_VEC_BLACK, pin_color: COLOR_VEC_GRAY, text: format!("Symmetry line #{}", id), tab: 0})
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
            let border_color = if self.selected == Selection::Object(id){
                COLOR_VEC_ORANGE
            } else if self.selected_shape_for_moving == Some(id) {
                COLOR_VEC_BLUE
            } else {
                COLOR_VEC_BLACK
            };
            entries.push(EntryInSideList{border_color: border_color, pin_color: color, text: write_object_label(id, obj), tab: 0})
        }
        for (&group_id, group) in &scene.area_groups {
            let group_is_selected_for_attaching_to_us = {
                if let Some(so) = self.selected_shape_for_attaching_to_group {
                    Some(group_id) == self.scene.objects[&so].group
                } else { false }
            };

            let group_color_id = scene.get_object_attrs(&scene.objects[group.perimeters.first().unwrap()].obj).color;
            let group_color = scene.colors[&group_color_id].clr;
            entries.push(EntryInSideList{
                border_color: if group_is_selected_for_attaching_to_us { COLOR_VEC_BLUE_2 } else { COLOR_VEC_BLACK },
                pin_color: group_color, text: format!("Area group #{}", group_id), tab: 0});

            for &area_obj_id in &group.perimeters {
                let obj = &scene.objects[&area_obj_id];
                entries.push(EntryInSideList{
                    border_color: if self.selected == Selection::Object(area_obj_id) {
                        COLOR_VEC_ORANGE
                    } else if Some(area_obj_id) == self.selected_shape_for_moving {
                        COLOR_VEC_BLUE
                    } else {
                        COLOR_VEC_BLACK
                    },
                    pin_color: group_color, text: write_object_label(area_obj_id, obj), tab: 1})
            }
        }
        self.draw_list_of_nodes(&entries, font);
    }
}

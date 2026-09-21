use macroquad::prelude::glam::*;

use std::time::*;
use std::ops::Bound::*;
use std::collections::{BTreeMap, BTreeSet};
use macroquad::input::{is_key_down, mouse_wheel, KeyCode};
use crate::drawing::{get_mouse_position_vec2, is_mouse_near_dot, is_no_mod_down, is_shift};
use crate::editor_visual::*;
use crate::embroidery::build_embroidery_image;
use crate::scene::*;
use crate::util::btreemap_usize_get_unused_id;

impl Editor {
    fn remove_object_from_its_group(&mut self, obj_id: usize, group_id: usize){
        let group = self.scene.area_groups.get_mut(&group_id).unwrap();
        group.perimeters.remove(&obj_id);
        if group.perimeters.is_empty() {
            self.scene.area_groups.remove(&group_id);
        }
    }

    fn delete_the_object_without_consequences(&mut self, id: usize){
        let sh = self.scene.objects.remove(&id).unwrap();
        if let Some(group_id) = sh.group {
            let group = self.scene.area_groups.get_mut(&group_id).unwrap();
            group.perimeters.remove(&id);
            if group.perimeters.is_empty(){
                self.scene.area_groups.remove(&group_id);
            }
        }
        let wash_option = |r: &mut Option<usize>|{
            if let Some(their_id) = r && *their_id == id {
                *r = None;
            }
        };
        if self.selected == Selection::Object(id){
            self.selected = Selection::Nothing;
        }
        wash_option(&mut self.selected_shape_for_attaching_to_group);
        wash_option(&mut self.selected_shape_for_moving);
        wash_option(&mut self.held_edited_perimeter);
        wash_option(&mut self.held_edited_thin_line);
        wash_option(&mut self.held_edited_thick_line);
    }

    pub fn delete_the_object(&mut self, id: usize, delete_original: bool){
        match &self.scene.objects[&id].obj {
            ObjectNode::RealObjectNode(obj) => {
                if let Some(clone_id) = obj.clone {
                    self.delete_the_object_without_consequences(clone_id);
                }
            },
            ObjectNode::GhostObject(ghost) => {
                if delete_original {
                    self.delete_the_object_without_consequences(ghost.source);
                }
            }
        }
        self.delete_the_object_without_consequences(id);
    }

    // Too bad we did not think of backward mapping
    fn get_ghosts_of_movement(&self, movement_id: usize) -> Vec<usize>{
        self.scene.objects.iter().filter_map(|(&id, node)|{
            if node.obj.is_ghost_object() { Some(id) } else {None}
        }).collect()
    }

    // Will cascade into object deletion
    pub fn delete_movement(&mut self, id: usize){
        self.scene.movements.remove(&id);
        for g_id in self.get_ghosts_of_movement(id){
            self.delete_the_object_without_consequences(g_id);
        }

        let wash_option = |r: &mut Option<usize>|{
            if let Some(their_id) = r && *their_id == id {
                *r = None;
            }
        };
        wash_option(&mut self.held_edited_sym);
        wash_option(&mut self.selected_movement_for_movement);
        if self.selected == Selection::Movement(id){
            self.selected = Selection::Nothing;
        }
    }

    pub fn get_points_mut_of_area(&mut self, area_id: usize) -> &mut Vec<Vec2> {
        &mut self.scene.objects.get_mut(&area_id).unwrap()
            .obj.as_real_object_node_mut().unwrap().att.shape.as_area_shape_mut().unwrap().points
    }

    pub fn get_points_mut_of_thin_line(&mut self, id: usize) -> &mut Vec<Vec2> {
        &mut self.scene.objects.get_mut(&id).unwrap()
            .obj.as_real_object_node_mut().unwrap().att.shape.as_line_shape_mut().unwrap().points
    }

    pub fn get_points_mut_of_thick_line(&mut self, id: usize) -> &mut Vec<Vec2> {
        &mut self.scene.objects.get_mut(&id).unwrap()
            .obj.as_real_object_node_mut().unwrap().att.shape.as_thick_line_shape_mut().unwrap().points
    }

    // Whatever we were drawing - it gets finished
    pub fn command_finish_drawing_progress(&mut self){
        if let Some(area_id) = self.held_edited_perimeter {
            let points = self.get_points_mut_of_area(area_id);
            if points.len() < 3 + 1 {
                self.delete_the_object(area_id, false);
            } else {
                points.pop();
                self.held_edited_perimeter = None;
            }
        } else if let Some(id) = self.held_edited_thin_line {
            let points = self.get_points_mut_of_thin_line(id);
            if points.len() < 2 + 1 {
                self.delete_the_object(id, false);
            } else {
               points.pop();
                self.held_edited_thin_line = None;
            }
        } else if let Some(id) = self.held_edited_thick_line {
            let points = self.get_points_mut_of_thick_line(id);
            if points.len() < 2 + 1 {
                self.delete_the_object(id, false);
            } else {
                points.pop();
                self.held_edited_thick_line = None;
            }
        } else if let Some(id) = self.held_edited_sym {
            self.delete_movement(id);
        }
    }

    fn click_in_drawing_mode(&mut self, is_gap: bool){
        if !self.mode.is_draw() {
            return;
        }
        let p = self.cam.get_mouse_scene_pos();
        match self.selected_tool {
            EditorDrawingTool::Symmetry => {
                if self.held_edited_sym.is_some(){
                    self.held_edited_sym = None;
                } else {
                    let id = btreemap_usize_get_unused_id(&self.scene.movements);
                    self.scene.movements.insert(id, MovementNode::SymmetryMovement(SymmetryMovement{
                        x: p.x, pos_y1: p.y, pos_y2: p.y
                    }));
                    self.held_edited_sym = Some(id);
                }
            }
            EditorDrawingTool::Area => {
                if let Some(cur_edited_id) = self.held_edited_perimeter {
                    let first = *self.get_points_mut_of_area(cur_edited_id).first().unwrap();
                    if self.cam.is_mouse_near_scene_dot(first) {
                        self.command_finish_drawing_progress();
                    } else {
                        let last = *self.get_points_mut_of_area(cur_edited_id).last().unwrap();
                        self.get_points_mut_of_area(cur_edited_id).push(last);
                    }
                } else {
                    let working_color_id = match self.selected_color { Some(x) => x, None => return };
                    let id = btreemap_usize_get_unused_id(&self.scene.objects);
                    let new_group_id = btreemap_usize_get_unused_id(&self.scene.area_groups);
                    self.scene.area_groups.insert(new_group_id, AreaShapeGroup{
                        control_center_pos: p, control_fill_dir_offset: vec2(1., 0.),
                        f_params : AreaDoubleFillParams::default(),
                        perimeters: BTreeSet::from([id])
                    });
                    self.scene.objects.insert(id, GroupedObjectNode{
                        obj: ObjectNode::RealObjectNode(RealObjectNode{
                            att: RealObjectAttrs{ shape: Shape::AreaShape(AreaShape {
                                points: vec![p, p], is_gap
                            }), color: working_color_id },
                            clone: None
                        }), group: Some(new_group_id)});
                    self.held_edited_perimeter = Some(id);
                }
            }
            EditorDrawingTool::ThinLine => {
                if let Some(edited_line_id) = self.held_edited_thin_line {
                    let last = *self.get_points_mut_of_thin_line(edited_line_id).last().unwrap();
                    self.get_points_mut_of_thin_line(edited_line_id).push(last);
                } else {
                    let working_color_id = match self.selected_color { Some(x) => x, None => return };
                    let id = btreemap_usize_get_unused_id(&self.scene.objects);
                    self.scene.objects.insert(id, GroupedObjectNode {group: None,
                        obj: ObjectNode::RealObjectNode(RealObjectNode {clone: None,
                            att: RealObjectAttrs {
                                shape: Shape::LineShape(LineShape{points: vec![p, p]}),
                                color: working_color_id }
                        })});
                    self.held_edited_thin_line = Some(id);
                }
            }
            EditorDrawingTool::ThickLine => {
                if let Some(edited_line_id) = self.held_edited_thick_line {
                    let last = *self.get_points_mut_of_thick_line(edited_line_id).last().unwrap();
                    self.get_points_mut_of_thick_line(edited_line_id).push(last);
                } else {
                    let working_color_id = match self.selected_color { Some(x) => x, None => return };
                    let id = btreemap_usize_get_unused_id(&self.scene.objects);
                    self.scene.objects.insert(id, GroupedObjectNode {group: None,
                        obj: ObjectNode::RealObjectNode(RealObjectNode {clone: None,
                            att: RealObjectAttrs {
                                shape: Shape::ThickLineShape(ThickLineShape{
                                    thickness: 2., points: vec![p, p], prolonged_tips: false,
                                    cross_dist: 0.33
                                }),
                                color: working_color_id }
                        })});
                    self.held_edited_thick_line = Some(id);
                }
            }
        }
    }

    pub fn command_drawing_mode_click_normal(&mut self){
        self.click_in_drawing_mode(false);
    }

    pub fn command_drawing_mode_click_negative(&mut self){
        if self.selected_tool != EditorDrawingTool::Area{
            return;
        }
        self.click_in_drawing_mode(true);
    }

    pub fn command_embroidery_config_control_click_normal(&mut self){
        if self.mode.is_embroidery(){
            for (&group_id, group) in &self.scene.area_groups {
                let (a, b) = self.get_area_group_control_arrow_scr_pos(group);
                if is_mouse_near_dot(a) {
                    self.held_arrow_start_embroidery_control_group = Some(group_id);
                    break;
                }
                if is_mouse_near_dot(b) {
                    self.held_arrow_end_embroidery_control_group = Some(group_id);
                    break;
                }
            }
        }
    }

    // We are talking about the same button that triggered `command_embroidery_config_control_click_normal`
    pub fn command_embroidery_config_control_pointer_btn_release(&mut self){
        self.held_arrow_start_embroidery_control_group = None;
        self.held_arrow_end_embroidery_control_group = None;
    }

    pub fn command_drawing_mode_selection_click(&mut self) {
        for (&obj_id, _) in &self.scene.objects {
            let points = self.scene.get_object_point_path(obj_id);
            if self.cam.is_mouse_near_path(&points){
                self.selected = Selection::Object(obj_id);
                break;
            }
        }
        for (&mov_id, mov) in &self.scene.movements {
            let (a, b) = match mov {
                MovementNode::SymmetryMovement(sym) => (vec2(sym.x, sym.pos_y1), vec2(sym.x, sym.pos_y2))
            };
            if self.cam.is_mouse_near_segment(a, b) {
                self.selected = Selection::Movement(mov_id);
                break
            }
        }
    }



    fn change_tool(&mut self, tool: EditorDrawingTool){
        if self.selected_tool != tool {
            self.command_finish_drawing_progress();
            self.selected_tool = tool;
        }
    }

    pub fn command_change_tool_to_area(&mut self) {
        self.change_tool(EditorDrawingTool::Area);
    }

    pub fn command_change_tool_to_thin_line(&mut self) {
        self.change_tool(EditorDrawingTool::ThinLine);
    }

    pub fn command_change_tool_to_thick_line(&mut self) {
        self.change_tool(EditorDrawingTool::ThickLine);
    }

    pub fn command_change_tool_to_sym(&mut self) {
        self.change_tool(EditorDrawingTool::Symmetry);
    }

    pub fn command_cancellation(&mut self){
        if let Some(area_id) = self.held_edited_perimeter {
            self.delete_the_object(area_id, false);
        } else if let Some(id) = self.held_edited_thin_line {
            self.delete_the_object(id, false);
        } else if let Some(id) = self.held_edited_thick_line {
            self.delete_the_object(id, false);
        } else if self.selected != Selection::Nothing {
            self.selected = Selection::Nothing;
        } else {
            self.selected_movement_for_movement = None;
            self.selected_shape_for_moving = None;
            self.selected_shape_for_attaching_to_group = None;
        }
    }

    pub fn perform_select_shape_for_reparenting(&mut self){
        if let Selection::Object(selected) = self.selected {
            if self.scene.get_object_attrs_by_id(selected).shape.is_area_shape(){
                self.selected_shape_for_attaching_to_group = Some(selected);
            }
        }
    }

    pub fn perform_selected_selected_for_movement_bind(&mut self){
        if let Selection::Object(selected) = self.selected {
            self.selected_shape_for_moving = Some(selected);
        } else if let Selection::Movement(selected) = self.selected {
            self.selected_movement_for_movement = Some(selected);
        }
    }

    pub fn perform_attaching_shape_to_group(&mut self){
        match (self.selected_shape_for_attaching_to_group, self.selected){
            (Some(par_id), Selection::Object(obj_id)) => {
                let me = &self.scene.objects[&obj_id];
                let my_attrs = self.scene.get_object_attrs(&me.obj);
                let par_attrs = self.scene.get_object_attrs_by_id(par_id);
                if my_attrs.shape.is_area_shape() &&par_attrs.color == my_attrs.color {
                    let old_group = me.group.unwrap();
                    let new_group = self.scene.objects[&par_id].group.unwrap();
                    if old_group != new_group {
                        self.remove_object_from_its_group(obj_id, old_group);
                        self.scene.area_groups.get_mut(&new_group).unwrap().perimeters.insert(obj_id);
                        self.scene.objects.get_mut(&obj_id).unwrap().group = Some(new_group);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn command_select_previous_color(&mut self) {
        self.selected_color = self.selected_color.and_then(|cur| {self.scene.colors.range(..cur).next_back()})
            .or_else(|| -> Option<(&usize, &UsedColor)> {self.scene.colors.last_key_value() })
            .map(|(&id,_): (&usize, &UsedColor)| {id});
    }

    pub fn command_select_next_color(&mut self) {
        self.selected_color = self.selected_color.and_then(|cur| {self.scene.colors.range((Excluded(cur), Unbounded)).next()})
            .or_else(|| {self.scene.colors.first_key_value() })
            .map(|(&id, _)|{id});
    }

    pub fn command_delete_selected(&mut self) {
        if let Selection::Object(obj_id) = self.selected {
            self.delete_the_object(obj_id, true);
        } else if let Selection::Movement(mov_id) = self.selected {
            self.delete_movement(mov_id);
        }
    }

    pub fn command_delete_selected_suppress_ghost_recoil(&mut self) {
        if let Selection::Object(obj_id) = self.selected {
            self.delete_the_object(obj_id, false);
        } else if let Selection::Movement(mov_id) = self.selected {
            self.delete_movement(mov_id);
        }
    }

    pub fn command_ack_pointer_motion(&mut self){
        if let Some(group) = self.held_arrow_start_embroidery_control_group {
            self.scene.area_groups.get_mut(&group).unwrap().control_center_pos = self.cam.screen_coord_to_scene(get_mouse_position_vec2());
        }

        if let Some(group_id) = self.held_arrow_end_embroidery_control_group {
            let group: &mut AreaShapeGroup = self.scene.area_groups.get_mut(&group_id).unwrap();
            let a = self.cam.scene_coord_to_screen(group.control_center_pos);
            let scr_dir = (get_mouse_position_vec2() - a).normalize();
            group.control_fill_dir_offset = vec2(scr_dir.x, -scr_dir.y);
        }

        if let Some(mov_id) = self.held_edited_sym {
            let x = self.scene.movements.get_mut(&mov_id).unwrap();
            x.as_symmetry_movement_mut().unwrap().pos_y2 = self.cam.get_mouse_scene_pos().y;
        }

        if let Some(area_id) = self.held_edited_perimeter {
            let p = self.cam.get_mouse_scene_pos();
            *self.get_points_mut_of_area(area_id).last_mut().unwrap() = p;
        }

        if let Some(thin_line_id) = self.held_edited_thin_line {
            let p = self.cam.get_mouse_scene_pos();
            *self.get_points_mut_of_thin_line(thin_line_id).last_mut().unwrap() = p;
        }

        if let Some(thick_line_id) = self.held_edited_thick_line {
            let p = self.cam.get_mouse_scene_pos();
            *self.get_points_mut_of_thick_line(thick_line_id).last_mut().unwrap() = p;
        }
    }

    pub fn command_ack_mouse_wheel_motion(&mut self){
        if mouse_wheel().1 != 0. {
            let s = 1.15f32.powf(mouse_wheel().1);
            let pointing = self.cam.screen_coord_to_scene(get_mouse_position_vec2());
            self.cam.top_left = pointing + (-pointing + self.cam.top_left) / s;
            self.cam.px_in_mm = self.cam.px_in_mm * s;
        }
    }

    pub fn command_ack_pressed_motion_keys(&mut self, t_previous: Instant, t_now: Instant, up_pressed: bool, left_pressed: bool, down_pressed: bool, right_pressed: bool){
        if is_no_mod_down() {
            let cam_movement_keys: &[bool] = &[up_pressed, left_pressed, down_pressed, right_pressed];
            if self.t_started_moving.is_none() && cam_movement_keys.iter().any(|&k|{k}) {
                self.t_started_moving = Some(t_now);
            } else if (self.t_started_moving.is_some() && cam_movement_keys.iter().all(|&k|{k})){
                self.t_started_moving = None;
            }
            if let Some(i_started_moving) = self.t_started_moving {
                let cam_pix_per_second = 800.0 + f32::min(0.5, (t_now - i_started_moving).as_secs_f32()) * 1000.;
                let delta = cam_pix_per_second / self.cam.px_in_mm * (t_now - t_previous).as_secs_f32();

                if up_pressed{
                    self.cam.top_left.y += delta;
                }
                if left_pressed{
                    self.cam.top_left.x -= delta;
                }
                if down_pressed{
                    self.cam.top_left.y -= delta;
                }
                if right_pressed{
                    self.cam.top_left.x += delta;
                }
            }
        }
    }

    pub fn command_put_object_in_its_own_area_group(&mut self){
        if !self.mode.is_draw(){
            return;
        }
        if let Selection::Object(sel_obj_id) = self.selected {
            if !self.scene.get_object_attrs_by_id(sel_obj_id).shape.is_area_shape(){
                return;
            }
            let old_group_id = self.scene.objects.get(&sel_obj_id).unwrap().group.unwrap();
            let new_group_id = btreemap_usize_get_unused_id(&self.scene.area_groups);
            self.remove_object_from_its_group(sel_obj_id, old_group_id);

            self.scene.area_groups.insert(new_group_id, AreaShapeGroup{
                perimeters: BTreeSet::from([sel_obj_id]), f_params: AreaDoubleFillParams::default(),
                control_center_pos: center_of_contour_of_points(&self.scene.get_object_point_path(sel_obj_id))
                , control_fill_dir_offset: vec2(1., 0.)
            });

            self.scene.objects.get_mut(&sel_obj_id).unwrap().group = Some(new_group_id);
        }
    }

    pub fn command_create_a_ghost_object(&mut self){
        if !self.mode.is_draw() { return; }
        match (self.selected_movement_for_movement, self.selected_shape_for_moving) {
            (Some(mov_id), Some(source_obj_id)) => {
                if let ObjectNode::RealObjectNode(node) = &self.scene.objects.get(&source_obj_id).unwrap().obj {
                    if node.clone.is_none(){
                        let clone_id = btreemap_usize_get_unused_id(&self.scene.objects);
                        self.scene.objects.insert(clone_id, GroupedObjectNode{
                            group: None, obj: ObjectNode::GhostObject(GhostObject{ movement: mov_id, source: source_obj_id})});
                        if self.scene.get_object_attrs_by_id(source_obj_id).shape.is_area_shape(){
                            let new_group_id = btreemap_usize_get_unused_id(&self.scene.area_groups);
                            self.scene.area_groups.insert(new_group_id, AreaShapeGroup{
                                perimeters: BTreeSet::from([clone_id]), f_params: AreaDoubleFillParams::default(),
                                control_center_pos: center_of_contour_of_points(&self.scene.get_object_point_path(clone_id)),
                                control_fill_dir_offset: vec2(1., 0.)
                            });
                            self.scene.objects.get_mut(&clone_id).unwrap().group = Some(new_group_id);
                        }
                    }
                }
            },
            _ => {}
        }
    }


    fn set_editor_mode(&mut self, mode: EditorMode){
        if self.mode != mode {
            self.command_finish_drawing_progress();
            self.mode = mode;
            if mode.is_stitch(){
                self.stitches = Some(build_embroidery_image(&self.scene));
                self.shown_stitches = self.get_total_stitch_number_in_shown_image();
            } else {
                self.stitches = None;
            }
        }
    }

    pub fn command_set_editor_mode_drawing(&mut self){
        self.set_editor_mode(EditorMode::Draw);
    }

    pub fn command_set_editor_mode_embroidery_config(&mut self){
        self.set_editor_mode(EditorMode::Embroidery);
    }

    pub fn command_set_editor_mode_stitch_image_viewing(&mut self){
        self.set_editor_mode(EditorMode::Stitch);
    }

    pub fn command_stitch_image_viewing_progress_back(&mut self, count: usize){
        if !self.mode.is_stitch() { return; }
        let new: i64 = (self.shown_stitches as i64) - (count as i64);
        self.shown_stitches = i64::max(0, new) as usize;
    }

    pub fn command_stitch_image_viewing_progress_forward(&mut self, count: usize){
        if !self.mode.is_stitch() { return; }
        let new = self.shown_stitches + count;
        self.shown_stitches = usize::min(self.get_total_stitch_number_in_shown_image(), new);
    }
}

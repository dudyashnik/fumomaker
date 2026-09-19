use macroquad::prelude::glam::*;

use std::collections::{BTreeMap, BTreeSet};
use crate::editor_visual::*;
use crate::scene::*;

impl Editor {
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
        wash_option(&mut self.selected_shape);
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
        wash_option(&mut self.selected_movement);
        wash_option(&mut self.selected_movement_for_movement);
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
    pub fn finish_drawing_progress(&mut self){
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

    pub fn change_tool(&mut self, tool: EditorDrawingTool){
        if self.selected_tool != tool {
            self.finish_drawing_progress();
            self.selected_tool = tool;
        }
    }
}
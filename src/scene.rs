use std::collections::{BTreeMap, BTreeSet};
use glam::*;
use enum_as_inner::EnumAsInner;
use serde::{Serialize, Deserialize};
use crate::{CoolColor, COOL_COLORS};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct SymmetryMovement{
    pub x: f32,
    pub pos_y1: f32,
    pub pos_y2: f32,
}

#[derive(EnumAsInner, Clone, Copy, Serialize, Deserialize)]
pub enum MovementNode{
    SymmetryMovement(SymmetryMovement),
}

impl MovementNode {
    pub fn forward(&self, p: Vec2) -> Vec2{
        match self {
            MovementNode::SymmetryMovement(sym) => {
                vec2(p.x + 2. * (-p.x + sym.x), p.y)
            }
        }
    }

    pub fn backward(&self, p: Vec2) -> Vec2 {
        match self {
            MovementNode::SymmetryMovement(sym) => {
                vec2(p.x + 2. * (-p.x + sym.x), p.y)
            }
        }
    }

    pub fn option_forward(opt: Option<Self>, p: Vec2) -> Vec2{
        match opt {
            Some(mov) => mov.forward(p), None => p
        }
    }

    pub fn option_backward(opt: Option<Self>, p: Vec2) -> Vec2 {
        match opt {
            Some(mov ) => mov.backward(p), None => p
        }
    }

    pub fn invert(opt: Self) -> Self {
        match opt { MovementNode::SymmetryMovement(sym) => MovementNode::SymmetryMovement(sym) }
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct UsedColor {
    pub clr: Vec3,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AreaShape {
    pub points: Vec<Vec2>,
    pub is_gap: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LineShape {
    pub points: Vec<Vec2>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ThickLineShape {
    pub points: Vec<Vec2>,
    pub thickness: f32,
    pub cross_dist: f32,
    pub prolonged_tips: bool,
}

#[derive(Clone, EnumAsInner, Serialize, Deserialize)]
pub enum Shape {
    AreaShape(AreaShape),
    LineShape(LineShape),
    ThickLineShape(ThickLineShape),
}

impl Shape {
    pub fn get_points(&self) -> &Vec<Vec2> {
        match self { Shape::AreaShape(a) => &a.points,
            Shape::ThickLineShape(a) => &a.points,
            Shape::LineShape(a) => &a.points,
        }
    }

    pub fn get_points_mut(&mut self) -> &mut Vec<Vec2> {
        match self { Shape::AreaShape(a) => &mut a.points,
            Shape::ThickLineShape(a) => &mut a.points,
            Shape::LineShape(a) => &mut a.points,
        }
    }

    pub fn get_point_path(&self) -> Vec<Vec2>{
        match self {
            Shape::AreaShape(area ) => {
                let mut p = area.points.clone();
                p.push(*p.first().unwrap()); p
            },
            Shape::ThickLineShape(a) => a.points.clone(),
            Shape::LineShape(a) => a.points.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RealObjectAttrs {
    pub shape: Shape,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RealObjectNode {
    pub att: RealObjectAttrs,
    pub clone: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GhostObject {
    pub movement: usize,
    pub source: usize,
}

#[derive(Clone, EnumAsInner, Serialize, Deserialize)]
pub enum ObjectNodeBase {
    RealObjectNode(RealObjectNode),
    GhostObject(GhostObject),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ColoredObjectNode {
    pub obj: ObjectNodeBase,
    pub color: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GroupedObjectNode {
    pub obj: ColoredObjectNode,
    pub group: Option<usize>,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AreaFillParams {
    pub fill_line_dist: f32,
    pub stitch_len: f32,
    pub stitch_phase_offset: f32,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AreaDoubleFillParams {
    pub primal_fill: AreaFillParams,
    pub hidden_fill: AreaFillParams,
    pub hopping_stitch_len: f32,
}

impl Default for AreaDoubleFillParams {
    fn default() -> Self { AreaDoubleFillParams {
        primal_fill: AreaFillParams {fill_line_dist: 0.35, stitch_len: 0.8, stitch_phase_offset: 0.5},
        hidden_fill: AreaFillParams {fill_line_dist: 0.45, stitch_len: 1.8, stitch_phase_offset: 0.33},
        hopping_stitch_len: 3.
    } }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AreaShapeGroup {
    pub control_center_pos: Vec2,
    pub control_fill_dir_offset: Vec2,
    pub f_params: AreaDoubleFillParams,
    pub perimeters: BTreeSet<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Scene {
    pub colors: BTreeMap<usize, UsedColor>,
    pub objects: BTreeMap<usize, GroupedObjectNode>,
    pub movements: BTreeMap<usize, MovementNode>,
    pub area_groups: BTreeMap<usize, AreaShapeGroup>
}

impl Default for Scene {
    fn default() -> Scene {
        Scene { colors: COOL_COLORS.iter().map(|&CoolColor{clr, name}|{ UsedColor{clr, name: name.to_string()} })
            .enumerate().collect(), objects: Default::default(),
            movements: Default::default(), area_groups: Default::default(), }
    }
}

impl Scene {
    pub fn get_object_attrs<'a>(&'a self, obj: &'a ColoredObjectNode) -> &'a RealObjectAttrs {
        match &obj.obj {
            ObjectNodeBase::RealObjectNode(ron) => &ron.att,
            ObjectNodeBase::GhostObject(gon) => &self.objects[&gon.source].obj.obj.as_real_object_node().unwrap().att
        }
    }

    pub fn get_object_attrs_by_id(&self, obj_id: usize) -> &RealObjectAttrs{
        self.get_object_attrs(&self.objects[&obj_id].obj)
    }

    pub fn get_source_id_and_transition_of_object_by_id(&self, obj_id: usize) -> (usize, Option<MovementNode>){
        match &self.objects[&obj_id].obj.obj {
            ObjectNodeBase::RealObjectNode(_) => (obj_id, None),
            ObjectNodeBase::GhostObject(ghost) => (ghost.source, Some(self.movements[&ghost.movement]))
        }
    }

    pub fn get_source_and_transition_of_object_node<'a>(&'a self, obj: &'a ColoredObjectNode) -> (&'a RealObjectAttrs, Option<MovementNode>){
        match &obj.obj {
            ObjectNodeBase::RealObjectNode(real) => (&real.att, None),
            ObjectNodeBase::GhostObject(ghost) => (
                &self.objects[&ghost.source].obj.obj.as_real_object_node().unwrap().att,
                Some(self.movements[&ghost.movement])
            )
        }
    }

    // Transforms them
    pub fn get_point_path_of_object_by_id(&self, obj_id: usize) -> Vec<Vec2> {
        let obj = &self.objects[&obj_id].obj;
        let (source, trans) = self.get_source_and_transition_of_object_node(obj);
        source.shape.get_point_path().iter().map(|&v|{ MovementNode::option_forward(trans, v) }).collect()
    }
}

pub fn center_of_contour_of_points(points: &[Vec2]) -> Vec2 {
    points.iter().sum::<Vec2>() / (points.len() as f32)
}
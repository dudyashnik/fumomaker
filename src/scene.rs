use std::collections::{BTreeMap, BTreeSet};
pub use macroquad::prelude::glam;
pub use glam::*;
use enum_as_inner::EnumAsInner;

#[derive(Clone, Copy)]
pub struct SymmetryMovement{
    pub x: f32,
    pub pos_y1: f32,
    pub pos_y2: f32,
}

#[derive(EnumAsInner, Clone, Copy)]
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
}


#[derive(Clone)]
pub struct UsedColor {
    pub clr: Vec3,
    pub name: String,
}

#[derive(Clone)]
pub struct AreaShape {
    pub points: Vec<Vec2>,
    pub is_gap: bool,
}

#[derive(Clone)]
pub struct LineShape {
    pub points: Vec<Vec2>,
}

#[derive(Clone)]
pub struct ThickLineShape {
    pub points: Vec<Vec2>,
    pub thickness: f32,
    pub prolonged_tips: bool,
}

#[derive(Clone, EnumAsInner)]
pub enum Shape {
    AreaShape(AreaShape),
    LineShape(LineShape),
    ThickLineShape(ThickLineShape),
}

#[derive(Clone)]
pub struct RealObjectAttrs {
    pub shape: Shape,
    pub color: usize,
}

#[derive(Clone)]
pub struct RealObjectNode {
    pub att: RealObjectAttrs,
    pub clone: Option<usize>,
}

#[derive(Clone)]
pub struct GhostObject {
    pub movement: usize,
    pub source: usize,
}

#[derive(Clone, EnumAsInner)]
pub enum ObjectNode {
    RealObjectNode(RealObjectNode),
    GhostObject(GhostObject),
}

#[derive(Clone)]
pub struct GroupedObjectNode {
    pub obj: ObjectNode,
    pub group: Option<usize>,
}

#[derive(Copy, Clone)]
pub struct AreaFillParams {
    pub fill_line_dist: f32,
    pub stitch_len: f32,
    pub stitch_phase_offset: f32,
}

#[derive(Copy, Clone)]
pub struct AreaDoubleFillParams {
    pub primal_fill: AreaFillParams,
    pub hidden_fill: AreaFillParams,
    pub hopping_stitch_len: f32,
}

impl Default for AreaDoubleFillParams {
    fn default() -> Self { AreaDoubleFillParams {
        primal_fill: AreaFillParams {fill_line_dist: 0.2, stitch_len: 0.8, stitch_phase_offset: 0.33},
        hidden_fill: AreaFillParams {fill_line_dist: 0.5, stitch_len: 1.6, stitch_phase_offset: 0.33},
        hopping_stitch_len: 1.9
    } }
}


#[derive(Clone)]
pub struct AreaShapeGroup {
    pub control_center_pos: Vec2,
    pub control_fill_dir_offset: Vec2,
    pub f_params: AreaDoubleFillParams,
    pub perimeters: BTreeSet<usize>,
}

#[derive(Clone)]
pub struct Scene {
    pub colors: BTreeMap<usize, UsedColor>,
    pub objects: BTreeMap<usize, GroupedObjectNode>,
    pub movements: BTreeMap<usize, MovementNode>,
    pub area_groups: BTreeMap<usize, AreaShapeGroup>
}

impl Default for Scene {
    fn default() -> Scene {
        Scene { colors: Default::default(), objects: Default::default(),
            movements: Default::default(), area_groups: Default::default(), }
    }
}

impl Scene {
    pub fn get_object_attrs<'a>(&'a self, obj: &'a ObjectNode) -> &'a RealObjectAttrs {
        match obj {
            ObjectNode::RealObjectNode(ron) => &ron.att,
            ObjectNode::GhostObject(gon) => &self.objects[&gon.source].obj.as_real_object_node().unwrap().att
        }
    }
}

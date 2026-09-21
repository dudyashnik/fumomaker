use std::collections::{BTreeSet, BTreeMap, HashSet, HashMap};
use std::iter::{IntoIterator, Iterator};
use enum_as_inner::EnumAsInner;

use glam::*;
use serde::{Deserialize, Serialize};

use crate::util::*;
use crate::scene::*;
use crate::trenches::*;

// From embroidery configuration mode to stitch display mode

#[derive(EnumAsInner, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum StitchKind { Normal, NormalBorder, HoppingInDescend, HoppingToTurnBack, HoppingStartOfJob, TransLevel }

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Stitch {
    pub end: Vec2,
    pub kind: StitchKind,
}

#[derive(Deserialize, Serialize)]
pub struct StitchPath {
    pub start: Vec2,
    pub stitches: Vec<Stitch>,
}

#[derive(Deserialize, Serialize)]
pub struct StitchColorGroup {
    pub color: UsedColor,
    pub paths: Vec<StitchPath>
}

#[derive(Deserialize, Serialize)]
pub struct EmbroideryImage {
    pub grp: Vec<StitchColorGroup>
}

pub fn fill_trench_zone(stitching_trenches: &TrenchZone, hopping_stitch_len: f32, params: AreaFillParams, suggested_start: Option<Vec2>) -> Option<StitchPath> {
    struct Vertex {
        my_y: usize,
        start_x: f32,
        end_x: f32,
        neighbours: [Vec<usize>;2],
        visited: bool,
    }

    let fm_pref : Vec<usize> = {
        let mut count: usize = 0;
        stitching_trenches.lines.iter().map(move |l|{
            count += l.len();
            count - l.len()
        }).collect()
    };

    let mut graph: Vec<Vertex> = stitching_trenches.lines.iter().enumerate().flat_map(|(y, trench)|{
        trench.iter().map(move |segment|{  Vertex {my_y: y, start_x: segment.start, end_x: segment.end,
            neighbours: [Vec::new(), Vec::new()], visited: false} })
    }).collect();

    for y_top in 1..stitching_trenches.lines.len() {
        let y_bottom = y_top - 1;
        struct SomeonesPoint {
            x: f32,
            is_top: bool,
            id_on_trench: usize
        }
        let mut com: Vec<SomeonesPoint> =
            [(&stitching_trenches.lines[y_bottom], false), (&stitching_trenches.lines[y_top], true)]
                .into_iter().flat_map(|(vec, is_top)| {
            vec.iter().enumerate().flat_map(move |(id_on_trench, seg)|{
                [SomeonesPoint{x: seg.start, is_top, id_on_trench}, SomeonesPoint{x: seg.end, is_top, id_on_trench }].into_iter()
            })
        }).collect();
        com.sort_by(|a, b| {a.x.partial_cmp(&b.x).unwrap()});
        let mut opened_bottom: Option<usize> = None;
        let mut opened_top: Option<usize> = None;
        for sp in com {
            if !sp.is_top {
                match opened_bottom {
                    None => { opened_bottom = Some(fm_pref[y_bottom] + sp.id_on_trench) },
                    Some(_) => { opened_bottom = None },
                }
            } else {
                match opened_top {
                    None => { opened_top = Some(fm_pref[y_top] + sp.id_on_trench) },
                    Some(_ ) => { opened_top = None },
                }
            }
            match (opened_bottom, opened_top) {
                (Some(of_bottom), Some(of_top)) => {
                    graph[of_bottom].neighbours[1].push(of_top);
                    graph[of_top].neighbours[0].push(of_bottom);
                }
                _ => {},
            }
        }
        assert_eq!(opened_bottom, None);
        assert_eq!(opened_top, None);
    }

    let mut res: Option<StitchPath> = None;
    if graph.is_empty(){
        return None;
    }

    #[derive(Copy, Clone)]
    enum YDir { Up, Down }

    impl YDir {
        fn ind(self) -> usize {
            match self { YDir::Up => 1, YDir::Down => 0, }
        }
        fn opposite(self) -> Self {
            match self { YDir::Up => YDir::Down, YDir::Down => YDir::Up, }
        }
        fn half_dist(self) -> f32 {
            match self { YDir::Up => 0.5, YDir::Down => -0.5, }
        }
    }

    #[derive(Clone, Copy)]
    enum StitchDir { Left, Right };

    impl StitchDir {
        fn opposite(self) -> Self {
            match self { StitchDir::Left => StitchDir::Right, StitchDir::Right => StitchDir::Left }
        }
    }

    /* Hopping chain can have two endings:
     * Start of stitch trench
     * Rotating to back_direction to go back. In that scenario we go in a parallel hopping trench
     * to reach the start of another hopping trench. Nothing scary
     */
    struct HoppingChain {
        y_dir: YDir,
        vertices: Vec<usize>,
        start_plane_x: f32,
    }

    struct St {
        in_progress_hopping: Option<HoppingChain>,
        done_something_significant: bool,

        start: Option<Vec2>,
        stitches: Vec<Stitch>,

        hopping_stitch_len: f32,
        params: AreaFillParams,
        a: Mat2,
        b: Vec2,
    }

    impl St {
        // I really mean 'to scene'
        fn to_screen(&self, x: f32, y: f32) -> Vec2 {
        self.b + self.a * vec2(x, y)
    }

        fn push_segment_stitch(&mut self, a: Vec2, b: Vec2, kind: StitchKind, stitch_length: f32){
            const BAD_LEN: f32 = 0.2;
            let to = (-a + b).normalize();
            let len = (-a + b).length();
            let sl = stitch_length;
            let mut p = sl;
            while p < len {
                if p + BAD_LEN > len {
                    break
                }
                self.stitches.push(Stitch {end: a + to * p, kind});
                p += sl;
            }
            self.stitches.push(Stitch {end: b, kind});
        }

        fn do_hopping_chain(&mut self, graph: &Vec<Vertex>, chain: HoppingChain){
            let mut prev_x = chain.start_plane_x;
            for chain_id in 1..chain.vertices.len() {
                let prev: &Vertex = &graph[chain.vertices[chain_id - 1]];
                let a = self.to_screen(prev_x, (prev.my_y as f32 - chain.y_dir.half_dist()) * self.params.fill_line_dist);
                if chain_id < chain.vertices.len() {
                    let next: &Vertex = &graph[chain.vertices[chain_id]];
                    let next_x = ((next.start_x + next.end_x) / 2.).clamp(prev.start_x, prev.end_x);
                    let b = self.to_screen(next_x, (next.my_y as f32 - chain.y_dir.half_dist()) * self.params.fill_line_dist);
                    self.push_segment_stitch(a, b, StitchKind::HoppingInDescend, self.hopping_stitch_len);
                    prev_x = next_x;
                } else {
                    let b = self.to_screen(prev_x, (prev.my_y as f32 + chain.y_dir.half_dist()) * self.params.fill_line_dist);
                    self.push_segment_stitch(a, b, StitchKind::HoppingInDescend, self.hopping_stitch_len);
                }
            }

        }

        fn end_hopping_chain(&mut self, graph: &Vec<Vertex>){
            match self.in_progress_hopping.take() {
                Some(chain) => {
                    self.do_hopping_chain(graph, chain);
                }
                _ => {}
            }
        }

        fn start_hopping_chain(&mut self, chain_start: &Vertex, y_dir: YDir) {
            assert!(self.in_progress_hopping.is_none());
            let x = (chain_start.start_x + chain_start.end_x) / 2.;
            let b = self.to_screen(x, (chain_start.my_y as f32 - y_dir.half_dist()) * self.params.fill_line_dist);
            if let Some(very_first_pos) = self.start {
                let a = self.stitches.last().map(|s|{s.end}).unwrap_or(very_first_pos);
                self.push_segment_stitch(a, b, StitchKind::HoppingToTurnBack, self.hopping_stitch_len);
            } else {
                self.start = Some(b);
            }
            self.in_progress_hopping = Some(HoppingChain {vertices: Vec::new(),
                start_plane_x: x, y_dir})
        }

        // Fills with (StitchKind::Normal, len = self.params.stitch_len). But there also will be
        // a customizable jump to the a-point of trench segment
        fn stitch_normal_trench(&mut self, v: &Vertex, x_dir: StitchDir, a_jump_kind: StitchKind, a_jump_stitch_length: f32) {
            assert!(self.in_progress_hopping.is_none());

            let to_screen = |st: &St, x: f32| -> Vec2 {
                st.to_screen(x, v.my_y as f32 * st.params.fill_line_dist)
            };
            const BAD_LEN: f32 = 0.2;

            let a = match x_dir { StitchDir::Left => v.end_x, StitchDir::Right => v.start_x };
            let b = match x_dir { StitchDir::Left => v.start_x, StitchDir::Right => v.end_x };
            let jump_init = self.stitches.last().map(|s| { s.end }).unwrap_or(self.start.unwrap());
            self.push_segment_stitch(jump_init, to_screen(self, a), a_jump_kind, a_jump_stitch_length);
            let len = (a - b).abs();
            let to = (-a + b).signum();
            assert_ne!(to, 0.);
            let offset = self.params.stitch_phase_offset * v.my_y as f32;
            let sl = self.params.stitch_len;
            let mut p = match x_dir {
                StitchDir::Right => ((v.start_x - offset) / sl).ceil() *sl + offset - v.start_x,
                StitchDir::Left => v.end_x - (((v.end_x - offset) / sl).floor() * sl + offset)
            };
            while p + BAD_LEN < len {
                if p > BAD_LEN {
                    self.stitches.push(Stitch {end: to_screen(self, a + to * p), kind: StitchKind::Normal})
                }
                p += sl;
            }
            self.stitches.push(Stitch {end: to_screen(self, b), kind: StitchKind::Normal});
            self.done_something_significant = true;
        }

    }

    fn dfs (v: usize, parent: Option<usize>, y_dir: YDir, x_dir: StitchDir, graph: &mut Vec<Vertex>, st: &mut St){
        if graph[v].visited {
            return
        }
        graph[v].visited = true;
        let dir_back: YDir = y_dir.opposite();
        let graph_c: &Vec<Vertex> = graph;
        let mut after_parent: Vec<usize> = graph_c[v].neighbours[dir_back.ind()].iter().copied().filter(|&bv|{
            if graph_c[bv].visited { return false }
            match parent {
                Some(p) => match x_dir {
                    StitchDir::Right => { bv < p }, StitchDir::Left => { p < bv }
                },
                None => false
            }
        }).collect();
        let mut before_parent: Vec<usize> = graph_c[v].neighbours[dir_back.ind()].iter().copied().filter(|&bv| {
            if graph_c[bv].visited { return false }
            match parent {
                Some(p) => match x_dir {
                    StitchDir::Right => { p < bv }, StitchDir::Left => {bv < p }
                },
                None => true
            }
        }).collect();
        let mut along_the_road: Vec<usize> = graph[v].neighbours[y_dir.ind()].iter().copied().filter(|av| {
            !graph[*av].visited
        }).collect();
        if let StitchDir::Right = &x_dir {
            after_parent.reverse();
            before_parent.reverse();
            along_the_road.reverse();
        }

        for &u in &after_parent {
            st.end_hopping_chain(graph);
            st.start_hopping_chain(&graph[u], dir_back);
            dfs(u, Some(v), dir_back, x_dir.opposite(), graph, st);
        }

        for (dri, &u) in along_the_road.iter().enumerate() {
            // Start hopping path on top of these fucks. Maybe, just maybe, we never ended our previous shit
            if dri == 0 && after_parent.is_empty() {
                // In that case we continue the shitting
                if st.in_progress_hopping.is_none(){
                    assert!(parent.is_none());
                    st.start_hopping_chain(&graph[u], y_dir);
                } else {
                    st.in_progress_hopping.as_mut().unwrap().vertices.push(v);
                }
            } else {
                st.start_hopping_chain(&graph[u], y_dir);
            }
            dfs(u, Some(v), y_dir, x_dir.opposite(), graph, st);
        }

        st.end_hopping_chain(graph);
        if along_the_road.is_empty() {
            st.stitch_normal_trench(&graph[v], x_dir, StitchKind::HoppingStartOfJob, st.hopping_stitch_len);
        } else {
            st.stitch_normal_trench(&graph[v], x_dir, StitchKind::NormalBorder, st.params.stitch_len);
        }

        for u in before_parent {
            // We place the hopping path here ( definitely a new one)
            st.end_hopping_chain(graph);
            st.start_hopping_chain(&graph[u], dir_back);
            dfs(u, Some(v), dir_back, x_dir.opposite(), graph, st);
        }
    }

    let mut st = St {done_something_significant: false,
        in_progress_hopping: None,
        start: None, stitches: Vec::new(), hopping_stitch_len, params,
        a: stitching_trenches.a, b: stitching_trenches.b};

    if graph.is_empty() { return None }

    let start_v = match suggested_start {
        None => {
            0
        },
        Some(old_end) => {
            let (closest_vert_id, closest_vert, closest_v_x, _): (usize, &Vertex, f32, f32) = graph
                .iter().enumerate().flat_map(|(vid, v): (usize, &Vertex)| {
                [(vid, v, v.start_x * 0.95 + v.end_x * 0.05), (vid, v, v.start_x * 0.05 + v.end_x * 0.95)].into_iter()
            }).map(|(vid, v, p): (usize, &Vertex, f32)| -> (usize, &Vertex, f32, f32){
                (vid, v, p, (st.to_screen(p, (v.my_y as f32 + 0.5) * st.params.fill_line_dist) - old_end).length())
            }).min_by(|(_, _, _, d1), (_, _, _, d2)|{ d1.partial_cmp(d2).unwrap() }).unwrap();
            st.start = Some(old_end);
            st.push_segment_stitch(old_end,
                                   st.to_screen(closest_v_x, (closest_vert.my_y as f32 + 0.5) * st.params.fill_line_dist),
                                   StitchKind::TransLevel, st.hopping_stitch_len);
            closest_vert_id
        }
    };
    dfs(start_v, None, YDir::Up, StitchDir::Right, &mut graph, &mut st);

    assert!(st.start.is_some());
    assert!(st.stitches.len() > 0);
    res = Some(StitchPath {start: st.start.unwrap(), stitches: st.stitches});
    res
}

impl StitchPath {
    fn from_normal_slice(points: &[Vec2]) -> Self{
        StitchPath {start: *points.first().unwrap(), stitches: points[1..].iter()
            .map(|&p|{Stitch{end: p, kind: StitchKind::Normal}}).collect()}
    }
}

pub fn stitch_path_for_thin_line(line: &LineShape) -> StitchPath {
    StitchPath::from_normal_slice(&line.points)
}

pub fn stitch_path_for_thick_line(line: &ThickLineShape) -> StitchPath {
    let d = line.thickness / 2.;
    let mut stitches: Vec<Stitch> = Vec::new();
    let get_to_the_left = |a: Vec2, b: Vec2| -> Vec2 {
        let dir = (-a + b).normalize();
        vec2(-dir.y, dir.x)
    };
    let mut points: Vec<Vec2> = Vec::new();
    let mut w = if line.prolonged_tips { -d } else { 0. };
    for i in 1..line.points.len() {
        let vb: Vec2 = line.points[i - 1];
        let vc: Vec2 = line.points[i];
        let full_l = (vb - vc).length();
        let end = full_l + if line.prolonged_tips && i == line.points.len() - 1 { d } else { 0. };
        while w < end{
            let left = {
                if w < d && i - 1 >= 1 {
                    let va = line.points[i - 2];
                    let coef_m = (w - (-d)) / (2. * d);
                    (get_to_the_left(va, vb) * (1. - coef_m) + get_to_the_left(vb, vc) * coef_m).normalize()
                } else if w > full_l - d && i + 1 < line.points.len() {
                    let vd = line.points[i + 1];
                    let coef_m = ((full_l + d) - w) / (2. * d);
                    (get_to_the_left(vb, vc) * coef_m + get_to_the_left(vc, vd) * (1. - coef_m)).normalize()
                } else {
                    get_to_the_left(vb, vc)
                }
            };
            let side = left * (if w < 0. {
                1. - (-w) / d
            } else if w > full_l {
                1. - (w - full_l) / d
            } else {
                1.
            });
            points.push(vb + (-vb+vc).normalize() * w - left);
            points.push(vb + (-vb+vc).normalize() * w + left);
            w += line.cross_dist;
        }
        w -= full_l;
    }
    StitchPath::from_normal_slice(&points)
}

pub fn stitch_path_for_area_shapes(scene: &Scene, perimeters: &BTreeSet<usize>, anchor_pos: Vec2, dir: Vec2, params: AreaDoubleFillParams) -> Option<StitchPath>{
    let (hidden_trench, primary_trench) = get_two_trench_zones(scene, perimeters, anchor_pos, dir, params);
    let mut res = None;
    let path_1_opt = fill_trench_zone(&hidden_trench, params.hopping_stitch_len,
                                  AreaFillParams { fill_line_dist: hidden_trench.dist, ..params.hidden_fill }, None);
    res = path_1_opt;
    let path_2_opt = fill_trench_zone(&primary_trench, params.hopping_stitch_len,
                                  AreaFillParams { fill_line_dist: primary_trench.dist, ..params.primal_fill },
                                  res.as_ref().map(|p|{ p.stitches.last().unwrap().end }));
    if let Some(mut path_2) = path_2_opt {
        if let Some(before) = &mut res{
            before.stitches.append(&mut path_2.stitches)
        } else {
            res = Some(path_2)
        }
    }

    // todo: something is fishy about it
    // if let Some(path) = &mut res {
    //     let i = path.stitches.iter().position(|stitch|{stitch.kind != StitchKind::HoppingInDescend}).unwrap();
    //     path.stitches.drain(..i);
    //     if i > 0 {
    //         path.start = path.stitches[i - 1].end;
    //     }
    // }
    res
}

pub fn build_embroidery_image(scene: &Scene) -> EmbroideryImage {
    let mut color_list: Vec<StitchColorGroup> = Vec::new();
    for (&color_id, color) in &scene.colors {
        let mut paths: Vec<StitchPath> = Vec::new();
        // Lines and Thick Lines (ungrouped shapes)
        for (&obj_id, g_obj) in &scene.objects{
            let (source, trans) = scene.get_source_and_transition_of_object_node(&g_obj.obj);
            if source.color != color_id { continue; }
            match &source.shape {
                Shape::AreaShape(_) => continue,
                Shape::LineShape(line) => {
                    let true_shape = LineShape{
                        points: line.points.iter().map(|&v|{ MovementNode::option_forward(trans, v) }).collect()
                    };
                    paths.push(stitch_path_for_thin_line(&true_shape));
                }
                Shape::ThickLineShape(line) => {
                    let true_shape = ThickLineShape{
                        points: line.points.iter().map(|&v|{ MovementNode::option_forward(trans, v) }).collect(),
                        ..*line
                    };
                    paths.push(stitch_path_for_thick_line(&true_shape));
                }
            }
        }

        // Area shapes
        for (&grp_id, group) in &scene.area_groups {
            if  scene.get_object_attrs_by_id(*group.perimeters.first().unwrap()).color != color_id {
                continue;
            }
            stitch_path_for_area_shapes(scene, &group.perimeters,
                    group.control_center_pos,
                    group.control_fill_dir_offset, group.f_params)
                .and_then(|path|{ paths.push(path); None::<()> });
        }

        if !paths.is_empty() {
            color_list.push(StitchColorGroup{color: color.clone(), paths});
        }
    }
    EmbroideryImage {grp: color_list}
}

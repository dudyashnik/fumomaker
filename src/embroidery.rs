use std::cmp::Ordering;
pub use super::scene::*;
use std::collections::{BTreeSet, BTreeMap, HashSet, HashMap};
use std::iter::{IntoIterator, Iterator};
use crate::embroidery::StitchKind::HoppingInDescend;
use super::util::*;

// From drawing mode to embroidery configuration mode

pub struct StitchingSegment {
    start: f32,
    end: f32,
}

pub struct TrenchZone {
    pub lines: Vec<Vec<StitchingSegment>>,
    pub dist: f32,
    pub a: Mat2,
    pub b: Vec2,
}

impl TrenchZone {
    fn to_scene(&self, x: f32, y: u32) -> Vec2 {
        self.b + self.a * vec2(x, y as f32 * self.dist)
    }
}

fn get_one_trench_zone(scene: &Scene, perimeters: &BTreeSet<usize>, anchor_pos: Vec2, dir: Vec2, params: AreaFillParams) -> TrenchZone {
    let dir = dir.normalize();
    struct Fixed {
        points: Vec<IVec2>,
        is_gap: bool,
    }
    // my coordinates, but precise
    fn to_discrete(v: f32) -> i32 {
        (v * 256.).round() as i32
    }
    fn to_discrete_vec2(v: Vec2) -> IVec2 {
        ivec2(to_discrete(v.x), to_discrete(v.y))
    }
    // my coordinates but imprecise
    fn to_floating(d: i64) -> f32 {
        (d as f32) / 256.
    }
    let scene_to_me = mat2(vec2(dir.y, dir.x), vec2(-dir.x, dir.y));
    let me_to_scene_rot = mat2(vec2(dir.y, -dir.x), vec2(dir.x, dir.y));
    let mut fixed: Vec<Fixed> = perimeters.iter().map(| perim_id: &usize| -> Option<Fixed> {
        let (source, trans): (&AreaShape, Option<MovementNode>) = match &scene.objects[perim_id].obj {
            ObjectNode::RealObjectNode(real) => (real.att.shape.as_area_shape().unwrap(), None),
            ObjectNode::GhostObject(ghost) => (
                scene.objects[&ghost.source].obj.as_real_object_node().unwrap().att.shape.as_area_shape().unwrap(),
                Some(scene.movements[&ghost.movement])
            )
        };
        if source.points.len() < 3 { return None }
        let mut points: Vec<IVec2> = Vec::new();
        for (i, cp) in source.points.iter().map(|v| {
            to_discrete_vec2(scene_to_me * MovementNode::option_forward(trans, *v) )
        }).enumerate(){
            if let Some(&pl) = points.last() && pl == cp {
                continue
            }
            if i == source.points.len() - 1 && points[0] == cp {
                continue
            }
            points.push(cp);
        }
        if points.len() < 3 { return None }
        Some(Fixed {points, is_gap: source.is_gap})
    }).flatten().collect();

    struct Sgeg {
        bottom: IVec2,
        top: IVec2,
        fixed_perim_id: usize,
    }

    let mut sgegs: Vec<Sgeg> = fixed.iter().enumerate().flat_map(|(fp_id, perim): (usize, &Fixed)| {
        perim.points.iter().zip(perim.points.iter().cycle().skip(1))
            .map(move |(a, b): (&IVec2, &IVec2)| -> Option<Sgeg> {
                let (bottom, top) = if a.y < b.y {(a, b)} else if b.y < a.y {(b, a)} else {
                    return None
                };
                Some(Sgeg {bottom: *bottom, top: *top, fixed_perim_id: fp_id})
            }).flatten()
    }).collect();

    struct Change {
        y: i32,
        sgeg_id: usize,
        is_end: bool,
    }

    let mut changes: Vec<Change> = sgegs.iter().enumerate().flat_map(|(sgeg_id, seg): (usize, &Sgeg)|{
        [ Change {y: seg.bottom.y, sgeg_id, is_end: false}, Change{y: seg.top.y, sgeg_id, is_end: true}  ].into_iter()
    }).collect();
    changes.sort_by(|a, b| {a.y.cmp(&b.y).then(a.is_end.cmp(&b.is_end))});

    if changes.is_empty() {
        return TrenchZone {lines: Vec::new(), dist: params.fill_line_dist, a: me_to_scene_rot, b: vec2(0., 0.)}
    }

    let bottom = changes.first().unwrap().y;
    let top = changes.last().unwrap().y;
    let trench_gap: i32 = i32::max(to_discrete(params.fill_line_dist), 1);
    let yy_known = to_discrete((scene_to_me * anchor_pos).y) + trench_gap / 2 - bottom;
    let yy_start = yy_known - yy_known.div_floor(trench_gap) * trench_gap;


    let mut trenches: Vec<Vec<StitchingSegment>> = Vec::new();
    let mut cur_change: usize = 0;
    let mut opened_sgegs: HashSet<usize> = HashSet::new();
    for my_y in (yy_start..top).step_by(trench_gap as usize){
        while let Some(nxt_change) = changes.get(cur_change) && nxt_change.y <= my_y {
            if (nxt_change.is_end){
                opened_sgegs.remove(&nxt_change.sgeg_id);
            } else {
                opened_sgegs.insert(nxt_change.sgeg_id);
            }
            cur_change += 1;
        }

        struct HorizontalChange {
            x: i64,
            fixed_area_id: usize,
        }
        let mut horizontal_changes: Vec<HorizontalChange> = opened_sgegs.iter().map(|id| -> HorizontalChange {
            let sgeg: &Sgeg = &sgegs[*id];
            let ax = sgeg.top.x as i64;
            let ay = sgeg.top.y as i64;
            let bx = sgeg.bottom.x as i64;
            let by = sgeg.bottom.y as i64;
            let x = ((ay - by) * bx - (ax - bx) * (by - my_y as i64)).div_ceil(ay - by);
            HorizontalChange {x, fixed_area_id: sgeg.fixed_perim_id}
        }).collect();
        horizontal_changes.sort_by(|a, b| {a.x.cmp(&b.x)});

        let mut activated_areas: Vec<bool> = Vec::new();
        activated_areas.resize(fixed.len(), false);
        let is_inside = 0i32;

        let mut last_opened: Option<i64> = None;
        let mut segments: Vec<StitchingSegment> = Vec::new();
        for hor_ch in horizontal_changes {
            let area: &Fixed = &fixed[hor_ch.fixed_area_id];
            let is_inside = is_inside +
                (if activated_areas[hor_ch.fixed_area_id] { -1i32 } else { 1i32 }) *
                (if area.is_gap { -1i32 } else { 1i32 });
            match last_opened {
                None if is_inside > 0 => {
                    last_opened = Some(hor_ch.x)
                },
                Some(start_x) if is_inside <= 0 => {
                    segments.push(StitchingSegment {start: start_x as f32, end: hor_ch.x as f32});
                    last_opened = None;
                }
                _ => {}
            }
            activated_areas[hor_ch.fixed_area_id] = !activated_areas[hor_ch.fixed_area_id];
        }
        assert!(last_opened.is_none());
        assert!(!activated_areas.iter().any(|x| { *x }));
        assert_eq!(is_inside, 0);
        trenches.push(segments);
    }
    assert!(opened_sgegs.is_empty());


    TrenchZone {lines: trenches, dist: to_floating(trench_gap as i64),
        a: me_to_scene_rot, b: dir * to_floating(yy_start as i64)}
}


fn get_two_trench_zones(scene: &Scene, perimeters: &BTreeSet<usize>, anchor_pos: Vec2, dir: Vec2, params: AreaDoubleFillParams) -> (TrenchZone, TrenchZone) {
    (
        get_one_trench_zone(scene, perimeters, anchor_pos, -vec2(-dir.y, dir.x), params.hidden_fill),
        get_one_trench_zone(scene, perimeters, anchor_pos, dir, params.primal_fill),
    )
}

// From embroidery configuration mode to stitch display mode

#[derive(Clone, Copy)]
pub enum StitchKind { Normal, NormalBorder, HoppingInDescend, HoppingToTurnBack, HoppingStartOfJob, TransLevel }

pub struct Stitch {
    pub end: Vec2,
    pub kind: StitchKind,
}

pub struct StitchPath {
    pub start: Vec2,
    pub stitches: Vec<Stitch>,
}

pub struct StitchColorGroup {
    pub color: usize,
    pub paths: Vec<StitchPath>
}

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

    for y_top in 1..(graph.len() - 1) {
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
        com.sort_by(|a, b| {a.x.partial_cmp(&a.x).unwrap()});
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

    struct ForgottenStartInfo {
        which_one_we_were_above: usize,
        which_x: f32
        /* Info about the point just above the which_one_we_were_above vertex:
         (which_x, (graph[which_one_we_were_above].y + 0.5) * line_dist )*/
    }

    struct St {
        in_progress_hopping: Option<HoppingChain>,
        forgotten_start: Option<ForgottenStartInfo>,
        done_something_significant: bool,

        start: Option<Vec2>,
        stitches: Vec<Stitch>,

        hopping_stitch_len: f32,
        params: AreaFillParams,
        a: Mat2,
        b: Vec2,
    }

    impl St {
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
                    for chain_id in 1..chain.vertices.len() {
                        if !self.done_something_significant {
                            if let Some(fs) = &self.forgotten_start {
                                if match chain.y_dir{
                                    YDir::Up => { fs.which_one_we_were_above == chain.vertices[chain_id] },
                                    YDir::Down => { fs.which_one_we_were_above == chain.vertices[chain_id - 1] },
                                } {
                                    loop {
                                        match self.stitches.last().map(|s|{s.kind}) {
                                            Some(StitchKind::HoppingInDescend) | Some(StitchKind::HoppingToTurnBack) => {self.stitches.pop();  },
                                            Some(StitchKind::HoppingStartOfJob) | Some(StitchKind::Normal) | Some(StitchKind::NormalBorder) => panic!(),
                                            _ => break
                                        }
                                    }
                                    self.do_hopping_chain(graph, HoppingChain{y_dir: chain.y_dir,
                                        start_plane_x: fs.which_x, vertices: chain.vertices[chain_id..].to_vec()  });
                                    return;
                                }
                            }
                        }
                    }
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
                st.in_progress_hopping.as_mut().unwrap().vertices.push(v);
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
            dfs(u, Some(v), dir_back, x_dir.opposite(), graph, st);
        }
    }

    let mut st = St {forgotten_start: None, done_something_significant: false,
        in_progress_hopping: None,
        start: None, stitches: Vec::new(), hopping_stitch_len, params,
        a: stitching_trenches.a, b: stitching_trenches.b};

    if graph.is_empty() { return None }

    let start_v = match suggested_start {
        None => {
            // This will be 100% ignored
            st.in_progress_hopping = Some(HoppingChain{y_dir: YDir::Up, vertices: Vec::new(),
                start_plane_x: (graph[0].start_x + graph[0].end_x) / 2.});
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
            st.forgotten_start = Some(ForgottenStartInfo{which_one_we_were_above: closest_vert_id, which_x: closest_v_x});
            st.push_segment_stitch(old_end,
                                   st.to_screen(closest_v_x, (closest_vert.my_y as f32 + 0.5) * st.params.fill_line_dist),
                                   StitchKind::TransLevel, st.hopping_stitch_len);
            st.in_progress_hopping = Some(HoppingChain{y_dir: YDir::Down, vertices: Vec::new(),
                start_plane_x: closest_v_x});
            let mut cur_v = closest_vert_id;
            loop {
                match graph[cur_v].neighbours[0].first() {
                    None => break,
                    Some(&lv) => {
                        st.in_progress_hopping.as_mut().unwrap().vertices.push(cur_v);
                        cur_v = lv;
                    }
                }
            }
            cur_v
        }
    };
    dfs(start_v, None,YDir::Up, StitchDir::Right, &mut graph, &mut st);

    assert!(st.start.is_some());
    assert!(st.stitches.len() > 0);
    res = Some(StitchPath {start: st.start.unwrap(), stitches: st.stitches});
    res
}

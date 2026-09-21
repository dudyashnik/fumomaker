use std::collections::{BTreeSet, BTreeMap, HashSet, HashMap};

use glam::*;
use crate::util::*;
use crate::scene::*;

// From drawing mode to embroidery configuration mode

pub struct StitchingSegment {
    pub start: f32,
    pub end: f32,
}

pub struct TrenchZone {
    pub lines: Vec<Vec<StitchingSegment>>,
    pub dist: f32,
    pub a: Mat2,
    pub b: Vec2,
}

pub fn get_one_trench_zone(scene: &Scene, perimeters: &BTreeSet<usize>, anchor_pos: Vec2, dir: Vec2, params: AreaFillParams) -> TrenchZone {
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
        let (source, trans): (&AreaShape, Option<MovementNode>) = {
            let (source_attrs, trans) = scene.get_source_and_transition_of_object_node(
                &scene.objects[perim_id].obj);
            (source_attrs.shape.as_area_shape().unwrap(), trans)
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
    let yy_known = to_discrete((scene_to_me * anchor_pos).y) + trench_gap / 2;
    let yy_start = yy_known + (-yy_known + bottom).div_ceil(trench_gap) * trench_gap;


    let mut trenches: Vec<Vec<StitchingSegment>> = Vec::new();
    let mut cur_change: usize = 0;
    let mut opened_sgegs: HashSet<usize> = HashSet::new();
    for my_y in (yy_start..top).step_by(trench_gap as usize){
        while let Some(nxt_change) = changes.get(cur_change) && nxt_change.y <= my_y {
            if nxt_change.is_end {
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
        let mut is_inside = 0i32;

        let mut last_opened: Option<i64> = None;
        let mut segments: Vec<StitchingSegment> = Vec::new();
        for hor_ch in horizontal_changes {
            let area: &Fixed = &fixed[hor_ch.fixed_area_id];
            is_inside = is_inside +
                (if activated_areas[hor_ch.fixed_area_id] { -1i32 } else { 1i32 }) *
                    (if area.is_gap { -1i32 } else { 1i32 });
            match last_opened {
                None if is_inside > 0 => {
                    last_opened = Some(hor_ch.x)
                },
                Some(start_x) if is_inside <= 0 => {
                    segments.push(StitchingSegment {start: to_floating(start_x), end: to_floating(hor_ch.x)});
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

    // Useless check
    while let Some(nxt_change) = changes.get(cur_change) {
        if nxt_change.is_end {
            opened_sgegs.remove(&nxt_change.sgeg_id);
        } else {
            opened_sgegs.insert(nxt_change.sgeg_id);
        }
        cur_change += 1;
    }
    assert!(opened_sgegs.is_empty());


    TrenchZone {lines: trenches, dist: to_floating(trench_gap as i64),
        a: me_to_scene_rot, b: dir * to_floating(yy_start as i64)}
}


pub fn get_two_trench_zones(scene: &Scene, perimeters: &BTreeSet<usize>, anchor_pos: Vec2, dir: Vec2, params: AreaDoubleFillParams) -> (TrenchZone, TrenchZone) {
    (
        get_one_trench_zone(scene, perimeters, anchor_pos, -vec2(-dir.y, dir.x), params.hidden_fill),
        get_one_trench_zone(scene, perimeters, anchor_pos, dir, params.primal_fill),
    )
}

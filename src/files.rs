use glam::*;

use crate::embroidery::*;
use crate::scene::*;
use crate::util::*;

use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;

pub fn save_scene_to_file(path: &str, scene: &Scene) -> BoxResult<()> {
    let text = serde_json::to_string::<Scene>(scene)?;
    fs::write(path, text)?;
    Ok(())
}

pub fn load_scene_from_file(path: &str) -> BoxResult<Scene> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<Scene>(&text)?)
}

type EmbroideryFileStitchData = (f32, f32, usize);

#[derive(Serialize, Deserialize)]
struct EmbroideryFileStitchPathData {
    start: Vec2,
    stitches: Vec<EmbroideryFileStitchData>,
}

#[derive(Serialize, Deserialize)]
struct EmbroideryFileStitchColorGroupData{
    paths: Vec<EmbroideryFileStitchPathData>,
    color: UsedColor,
}

type EmbroideryFileImageData = Vec<EmbroideryFileStitchColorGroupData>;

pub fn save_embroidery_image_to_file(path: &str, img: &EmbroideryImage) -> BoxResult<()> {
    let converted: EmbroideryFileImageData = img.grp.iter().map(|mg|{ EmbroideryFileStitchColorGroupData{
        color: mg.color.clone(),
        paths: mg.paths.iter().map(|mp|{ EmbroideryFileStitchPathData{
            start: mp.start,
            stitches: mp.stitches.iter().map(|ms|{ (ms.end.x, ms.end.y, match ms.kind {
                StitchKind::Normal => 0,
                StitchKind::NormalBorder => 1,
                StitchKind::HoppingInDescend => 2,
                StitchKind::HoppingToTurnBack => 3,
                StitchKind::HoppingStartOfJob => 4,
                StitchKind::TransLevel => 5,
            }) }).collect(),
        } }).collect(),
    } }).collect();
    let text = serde_json::to_string(&converted)?;
    fs::write(path, text)?;
    Ok(())
}
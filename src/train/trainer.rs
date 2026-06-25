use crate::{train::colmap::ColmapScene, utils::triangle_splat::TriangleSplat};

pub struct Trainer<const K: usize> {
    colmap_scene: ColmapScene,
    triangles: Vec<TriangleSplat<K>>
}

impl<const K: usize> Trainer<K> {
    pub fn init(colmap_scene_path: &str) {
        let colmap_scene: ColmapScene = ColmapScene::new(colmap_scene_path);
    }
}

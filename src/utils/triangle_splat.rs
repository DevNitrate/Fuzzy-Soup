use bevy::math::Vec3;

use crate::utils::spherical_voronoi::SphericalVoronoi;

pub struct TriangleSplat<const K: usize> {
    pub vertices: [Vec3; 3],
    pub color: SphericalVoronoi<K>,
    pub opacity: f32,
    pub sigma: f32
}

use bevy::math::Vec3;

pub struct SphericalVoronoi<const K: usize> {
    site_dirs: [Vec3; K],
    site_colors: [Vec3; K],
    log_tau: [f32; K]
}

mod aabb;
mod frustum;
mod plane;

pub use aabb::*;
pub use frustum::*;
pub use plane::*;

use lumi_util::math::Vec3A;

pub trait BoundingShape {
    fn center(&self) -> Vec3A;

    fn relative_radius(&self, normal: Vec3A, axes: &[Vec3A; 3]) -> f32;
}

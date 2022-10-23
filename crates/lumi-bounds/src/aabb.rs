use lumi_util::math::Vec3A;

use crate::BoundingShape;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub min: Vec3A,
    pub max: Vec3A,
}

impl Aabb {
    pub const ZERO: Self = Self::new(Vec3A::ZERO, Vec3A::ZERO);

    #[inline]
    pub const fn new(min: Vec3A, max: Vec3A) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn add_point(&mut self, point: impl Into<Vec3A>) {
        let point = point.into();

        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    #[inline]
    pub fn center(&self) -> Vec3A {
        (self.min + self.max) / 2.0
    }

    #[inline]
    pub fn half_extents(&self) -> Vec3A {
        (self.max - self.min) / 2.0
    }

    #[inline]
    pub fn to_center_half_extends(&self) -> (Vec3A, Vec3A) {
        (self.center(), self.half_extents())
    }
}

impl BoundingShape for Aabb {
    #[inline]
    fn center(&self) -> Vec3A {
        self.center()
    }

    #[inline]
    fn relative_radius(&self, normal: Vec3A, axes: &[Vec3A; 3]) -> f32 {
        let half_extents = self.half_extents();

        let v = Vec3A::new(
            normal.dot(axes[0]),
            normal.dot(axes[1]),
            normal.dot(axes[2]),
        );

        v.abs().dot(half_extents)
    }
}

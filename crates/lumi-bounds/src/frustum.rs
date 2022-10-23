use lumi_util::math::{Mat4, Vec3, Vec3A};

use crate::{BoundingShape, Plane};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    #[inline]
    pub const fn new(planes: [Plane; 6]) -> Self {
        Self { planes }
    }

    #[inline]
    pub fn from_view_proj(view: Mat4, proj: Mat4, far: f32) -> Self {
        let view_translation = view.w_axis.truncate();
        let view_backwards = view.transform_vector3(Vec3::Z);

        let view_proj = proj * view.inverse();

        let row3 = view_proj.row(3);

        let mut planes = [Plane::default(); 6];
        for (i, plane) in planes.iter_mut().enumerate().take(5) {
            let row = view_proj.row(i / 2);
            if (i & 1) == 0 && i != 4 {
                *plane = Plane::from_normal_d(row3 + row);
            } else {
                *plane = Plane::from_normal_d(row3 - row);
            }
        }

        let far_center = view_translation - view_backwards * far;
        planes[5] = Plane::new(view_backwards, -view_backwards.dot(far_center));
        Self::new(planes)
    }

    #[inline]
    pub fn planes(&self) -> &[Plane; 6] {
        &self.planes
    }

    #[inline]
    pub fn planes_mut(&mut self) -> &mut [Plane; 6] {
        &mut self.planes
    }

    #[inline]
    pub fn contains_shape<T: BoundingShape>(
        &self,
        shape: &T,
        transform: Mat4,
        intersect_far: bool,
    ) -> bool {
        let center = shape.center();
        let aabb_center_world = transform.transform_point3a(center.into()).extend(1.0);
        let axes = [
            Vec3A::from(transform.x_axis),
            Vec3A::from(transform.y_axis),
            Vec3A::from(transform.z_axis),
        ];

        let max = if intersect_far { 6 } else { 5 };
        for plane in &self.planes[..max] {
            let p_normal = plane.normal();
            let relative_radius = shape.relative_radius(p_normal, &axes);

            if plane.normal_d().dot(aabb_center_world) - relative_radius <= 0.0 {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn intersects_shape<T: BoundingShape>(
        &self,
        shape: &T,
        transform: Mat4,
        intersect_far: bool,
    ) -> bool {
        let center = shape.center();
        let aabb_center_world = transform.transform_point3a(center.into()).extend(1.0);
        let axes = [
            Vec3A::from(transform.x_axis),
            Vec3A::from(transform.y_axis),
            Vec3A::from(transform.z_axis),
        ];

        let max = if intersect_far { 6 } else { 5 };
        for plane in &self.planes[..max] {
            let p_normal = plane.normal();
            let relative_radius = shape.relative_radius(p_normal, &axes);

            if plane.normal_d().dot(aabb_center_world) + relative_radius <= 0.0 {
                return false;
            }
        }

        true
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraFrustum {
    pub frustum: Frustum,
    pub view: Mat4,
    pub proj: Mat4,
    pub far: Option<f32>,
}

impl CameraFrustum {
    #[inline]
    pub const fn intersect_far(&self) -> bool {
        self.far.is_none()
    }

    #[inline]
    pub fn contains_shape<T: BoundingShape>(&self, shape: &T, transform: Mat4) -> bool {
        self.frustum
            .contains_shape(shape, transform, self.intersect_far())
    }

    #[inline]
    pub fn intersects_shape<T: BoundingShape>(&self, shape: &T, transform: Mat4) -> bool {
        self.frustum
            .intersects_shape(shape, transform, self.intersect_far())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CascadeFrustum {
    pub outer: Frustum,
    pub inner: Option<Frustum>,
}

impl CascadeFrustum {
    #[inline]
    pub const fn intersect_far(&self) -> bool {
        true
    }

    #[inline]
    pub fn intersects_shape<T: BoundingShape>(&self, shape: &T, transform: Mat4) -> bool {
        let intersect_far = self.intersect_far();

        if let Some(inner) = self.inner {
            self.outer.intersects_shape(shape, transform, intersect_far)
                && !inner.contains_shape(shape, transform, intersect_far)
        } else {
            self.outer.intersects_shape(shape, transform, intersect_far)
        }
    }
}

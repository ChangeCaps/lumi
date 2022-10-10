use glam::{Mat4, Vec3, Vec3A, Vec4};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub min: Vec3A,
    pub max: Vec3A,
}

impl Aabb {
    pub const ZERO: Self = Self::new(Vec3A::ZERO, Vec3A::ZERO);

    pub const fn new(min: Vec3A, max: Vec3A) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn add_point(&mut self, point: Vec3) {
        self.min = self.min.min(point.into());
        self.max = self.max.max(point.into());
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

    #[inline]
    pub fn relative_radius(&self, p_normal: Vec3A, axes: &[Vec3A; 3]) -> f32 {
        let half_extents = self.half_extents();

        let v = Vec3A::new(
            p_normal.dot(axes[0]),
            p_normal.dot(axes[1]),
            p_normal.dot(axes[2]),
        );

        v.abs().dot(half_extents)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane {
    normal_d: Vec4,
}

impl Plane {
    pub const fn new(normal: Vec3, d: f32) -> Self {
        Self {
            normal_d: Vec4::new(normal.x, normal.y, normal.z, d),
        }
    }

    pub const fn from_normal_d(normal_d: Vec4) -> Self {
        Self { normal_d }
    }

    #[inline]
    pub fn normal(&self) -> Vec3A {
        Vec3A::from(self.normal_d)
    }

    #[inline]
    pub fn d(&self) -> f32 {
        self.normal_d.w
    }

    #[inline]
    pub fn normal_d(&self) -> Vec4 {
        self.normal_d
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
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
    pub fn contains_obb(&self, aabb: &Aabb, transform: Mat4, intersect_far: bool) -> bool {
        let center = aabb.center();
        let aabb_center_world = transform.transform_point3a(center.into()).extend(1.0);
        let axes = [
            Vec3A::from(transform.x_axis),
            Vec3A::from(transform.y_axis),
            Vec3A::from(transform.z_axis),
        ];

        let max = if intersect_far { 6 } else { 5 };
        for plane in &self.planes[..max] {
            let p_normal = plane.normal();
            let relative_radius = aabb.relative_radius(p_normal, &axes);

            if plane.normal_d().dot(aabb_center_world) - relative_radius <= 0.0 {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn intersects_obb(&self, aabb: &Aabb, transform: Mat4, intersect_far: bool) -> bool {
        let center = aabb.center();
        let aabb_center_world = transform.transform_point3a(center.into()).extend(1.0);
        let axes = [
            Vec3A::from(transform.x_axis),
            Vec3A::from(transform.y_axis),
            Vec3A::from(transform.z_axis),
        ];

        let max = if intersect_far { 6 } else { 5 };
        for plane in &self.planes[..max] {
            let p_normal = plane.normal();
            let relative_radius = aabb.relative_radius(p_normal, &axes);

            if plane.normal_d().dot(aabb_center_world) + relative_radius <= 0.0 {
                return false;
            }
        }

        true
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderFrustum {
    pub intersects: Frustum,
    pub without: Option<Frustum>,
    pub intersect_far: bool,
}

impl RenderFrustum {
    pub fn should_render(&self, aabb: &Aabb, transform: Mat4) -> bool {
        if let Some(without) = self.without {
            if without.contains_obb(aabb, transform, self.intersect_far) {
                return false;
            }
        }

        self.intersects
            .intersects_obb(aabb, transform, self.intersect_far)
    }
}

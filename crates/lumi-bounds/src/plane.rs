use lumi_util::math::{Vec3, Vec3A, Vec4, Vec4Swizzles};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane {
    normal_d: Vec4,
}

impl Plane {
    #[inline]
    pub fn new(normal: Vec3, d: f32) -> Self {
        Self::from_normal_d(Vec4::new(normal.x, normal.y, normal.z, d))
    }

    #[inline]
    pub fn from_normal_d(normal_d: Vec4) -> Self {
        Self {
            normal_d: normal_d * normal_d.xyz().length_recip(),
        }
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

    #[inline]
    pub fn distance(&self, point: Vec3A) -> f32 {
        self.normal_d.dot(point.extend(1.0))
    }
}

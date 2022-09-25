use glam::{IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

#[derive(Clone, Debug)]
pub enum MeshAttribute {
    Float32(Vec<f32>),
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Sint32(Vec<i32>),
    Sint32x2(Vec<[i32; 2]>),
    Sint32x3(Vec<[i32; 3]>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32(Vec<u32>),
    Uint32x2(Vec<[u32; 2]>),
    Uint32x3(Vec<[u32; 3]>),
    Uint32x4(Vec<[u32; 4]>),
}

impl MeshAttribute {
    pub fn len(&self) -> usize {
        match self {
            MeshAttribute::Float32(v) => v.len(),
            MeshAttribute::Float32x2(v) => v.len(),
            MeshAttribute::Float32x3(v) => v.len(),
            MeshAttribute::Float32x4(v) => v.len(),
            MeshAttribute::Sint32(v) => v.len(),
            MeshAttribute::Sint32x2(v) => v.len(),
            MeshAttribute::Sint32x3(v) => v.len(),
            MeshAttribute::Sint32x4(v) => v.len(),
            MeshAttribute::Uint32(v) => v.len(),
            MeshAttribute::Uint32x2(v) => v.len(),
            MeshAttribute::Uint32x3(v) => v.len(),
            MeshAttribute::Uint32x4(v) => v.len(),
        }
    }

    pub fn data(&self) -> &[u8] {
        match self {
            Self::Float32(data) => bytemuck::cast_slice(data),
            Self::Float32x2(data) => bytemuck::cast_slice(data),
            Self::Float32x3(data) => bytemuck::cast_slice(data),
            Self::Float32x4(data) => bytemuck::cast_slice(data),
            Self::Sint32(data) => bytemuck::cast_slice(data),
            Self::Sint32x2(data) => bytemuck::cast_slice(data),
            Self::Sint32x3(data) => bytemuck::cast_slice(data),
            Self::Sint32x4(data) => bytemuck::cast_slice(data),
            Self::Uint32(data) => bytemuck::cast_slice(data),
            Self::Uint32x2(data) => bytemuck::cast_slice(data),
            Self::Uint32x3(data) => bytemuck::cast_slice(data),
            Self::Uint32x4(data) => bytemuck::cast_slice(data),
        }
    }
}

pub trait AsMeshAttribute {
    fn as_mesh_attribute(attribute: &MeshAttribute) -> Option<&Self>;
    fn as_mesh_attribute_mut(attribute: &mut MeshAttribute) -> Option<&mut Self>;
}

macro_rules! impl_as_mesh_attribute {
    ($type:ty, $variant:ident) => {
        impl AsMeshAttribute for $type {
            fn as_mesh_attribute(attribute: &MeshAttribute) -> Option<&Self> {
                match attribute {
                    MeshAttribute::$variant(value) => Some(value),
                    _ => None,
                }
            }

            fn as_mesh_attribute_mut(attribute: &mut MeshAttribute) -> Option<&mut Self> {
                match attribute {
                    MeshAttribute::$variant(value) => Some(value),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! impl_as_mesh_attribute_vec {
    ($type:ty, $variant:ident) => {
        impl AsMeshAttribute for $type {
            fn as_mesh_attribute(attribute: &MeshAttribute) -> Option<&Self> {
                match attribute {
                    MeshAttribute::$variant(value) => Some(bytemuck::cast_slice(value)),
                    _ => None,
                }
            }

            fn as_mesh_attribute_mut(attribute: &mut MeshAttribute) -> Option<&mut Self> {
                match attribute {
                    MeshAttribute::$variant(value) => Some(bytemuck::cast_slice_mut(value)),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! impl_into_mesh_attribute {
    ($type:ty, $variant:ident) => {
        impl From<$type> for MeshAttribute {
            fn from(value: $type) -> Self {
                MeshAttribute::$variant(bytemuck::allocation::cast_vec(value))
            }
        }

        impl From<MeshAttribute> for $type {
            fn from(attribute: MeshAttribute) -> Self {
                match attribute {
                    MeshAttribute::$variant(value) => bytemuck::allocation::cast_vec(value),
                    _ => panic!("invalid mesh attribute"),
                }
            }
        }
    };
}

impl_as_mesh_attribute!(Vec<f32>, Float32);
impl_as_mesh_attribute!(Vec<[f32; 2]>, Float32x2);
impl_as_mesh_attribute!(Vec<[f32; 3]>, Float32x3);
impl_as_mesh_attribute!(Vec<[f32; 4]>, Float32x4);
impl_into_mesh_attribute!(Vec<f32>, Float32);
impl_into_mesh_attribute!(Vec<[f32; 2]>, Float32x2);
impl_into_mesh_attribute!(Vec<[f32; 3]>, Float32x3);
impl_into_mesh_attribute!(Vec<[f32; 4]>, Float32x4);
impl_into_mesh_attribute!(Vec<Vec2>, Float32x2);
impl_into_mesh_attribute!(Vec<Vec3>, Float32x3);
impl_into_mesh_attribute!(Vec<Vec4>, Float32x4);

impl_as_mesh_attribute!([f32], Float32);
impl_as_mesh_attribute!([[f32; 2]], Float32x2);
impl_as_mesh_attribute!([[f32; 3]], Float32x3);
impl_as_mesh_attribute!([[f32; 4]], Float32x4);
impl_as_mesh_attribute_vec!([Vec2], Float32x2);
impl_as_mesh_attribute_vec!([Vec3], Float32x3);
impl_as_mesh_attribute_vec!([Vec4], Float32x4);

impl_as_mesh_attribute!(Vec<i32>, Sint32);
impl_as_mesh_attribute!(Vec<[i32; 2]>, Sint32x2);
impl_as_mesh_attribute!(Vec<[i32; 3]>, Sint32x3);
impl_as_mesh_attribute!(Vec<[i32; 4]>, Sint32x4);
impl_into_mesh_attribute!(Vec<i32>, Sint32);
impl_into_mesh_attribute!(Vec<[i32; 2]>, Sint32x2);
impl_into_mesh_attribute!(Vec<[i32; 3]>, Sint32x3);
impl_into_mesh_attribute!(Vec<[i32; 4]>, Sint32x4);
impl_into_mesh_attribute!(Vec<IVec2>, Sint32x2);
impl_into_mesh_attribute!(Vec<IVec3>, Sint32x3);
impl_into_mesh_attribute!(Vec<IVec4>, Sint32x4);

impl_as_mesh_attribute!([i32], Sint32);
impl_as_mesh_attribute!([[i32; 2]], Sint32x2);
impl_as_mesh_attribute!([[i32; 3]], Sint32x3);
impl_as_mesh_attribute!([[i32; 4]], Sint32x4);
impl_as_mesh_attribute_vec!([IVec2], Sint32x2);
impl_as_mesh_attribute_vec!([IVec3], Sint32x3);
impl_as_mesh_attribute_vec!([IVec4], Sint32x4);

impl_as_mesh_attribute!(Vec<u32>, Uint32);
impl_as_mesh_attribute!(Vec<[u32; 2]>, Uint32x2);
impl_as_mesh_attribute!(Vec<[u32; 3]>, Uint32x3);
impl_as_mesh_attribute!(Vec<[u32; 4]>, Uint32x4);
impl_into_mesh_attribute!(Vec<u32>, Uint32);
impl_into_mesh_attribute!(Vec<[u32; 2]>, Uint32x2);
impl_into_mesh_attribute!(Vec<[u32; 3]>, Uint32x3);
impl_into_mesh_attribute!(Vec<[u32; 4]>, Uint32x4);
impl_into_mesh_attribute!(Vec<UVec2>, Uint32x2);
impl_into_mesh_attribute!(Vec<UVec3>, Uint32x3);
impl_into_mesh_attribute!(Vec<UVec4>, Uint32x4);

impl_as_mesh_attribute!([u32], Uint32);
impl_as_mesh_attribute!([[u32; 2]], Uint32x2);
impl_as_mesh_attribute!([[u32; 3]], Uint32x3);
impl_as_mesh_attribute!([[u32; 4]], Uint32x4);
impl_as_mesh_attribute_vec!([UVec2], Uint32x2);
impl_as_mesh_attribute_vec!([UVec3], Uint32x3);
impl_as_mesh_attribute_vec!([UVec4], Uint32x4);

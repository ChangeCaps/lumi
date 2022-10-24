use lumi_util::math::Vec3;

use crate::Mesh;

impl Mesh {
    pub fn generate_normals(&mut self) {
        if let Some(positions) = self.attribute::<[Vec3]>(Self::POSITION) {
            let mut normals: Vec<Vec3> = Vec::with_capacity(positions.len());

            if let Some(indices) = self.indices() {
                for i in (0..indices.len()).step_by(3) {
                    let a = positions[indices[i + 0] as usize];
                    let b = positions[indices[i + 1] as usize];
                    let c = positions[indices[i + 2] as usize];

                    let normal = (b - a).cross(c - a).normalize();

                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                }
            } else {
                for i in (0..positions.len()).step_by(3) {
                    let a = positions[i + 0];
                    let b = positions[i + 1];
                    let c = positions[i + 2];

                    let normal = (b - a).cross(c - a).normalize();

                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                }
            }

            for normal in normals.iter_mut() {
                *normal = normal.normalize();
            }

            self.insert_attribute(Self::NORMAL, normals);
        }
    }

    pub fn with_normals(mut self) -> Self {
        if !self.has_attribute(Self::NORMAL) {
            self.generate_normals();
        }

        self
    }
}

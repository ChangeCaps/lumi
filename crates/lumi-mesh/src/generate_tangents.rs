use crate::Mesh;

impl Mesh {
    pub fn generate_tangents(&mut self) -> bool {
        if !self.has_attribute(Self::POSITION)
            || !self.has_attribute(Self::NORMAL)
            || !self.has_attribute(Self::UV_0)
        {
            return false;
        }

        let tangents = vec![[0.0; 4]; self.attribute_len(Self::POSITION)];
        self.insert_attribute(Self::TANGENT, tangents);

        mikktspace::generate_tangents(self)
    }

    pub fn with_tangents(mut self) -> Self {
        self.generate_tangents();
        self
    }
}

impl mikktspace::Geometry for Mesh {
    fn num_faces(&self) -> usize {
        match self.indices() {
            Some(indices) => indices.len() / 3,
            None => self.attribute_len(Self::POSITION) / 3,
        }
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute::<[[f32; 3]]>(Self::POSITION).unwrap()[index]
            }
            None => self.attribute::<[[f32; 3]]>(Self::POSITION).unwrap()[face * 3 + vert],
        }
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute::<[[f32; 3]]>(Self::NORMAL).unwrap()[index]
            }
            None => self.attribute::<[[f32; 3]]>(Self::NORMAL).unwrap()[face * 3 + vert],
        }
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute::<[[f32; 2]]>(Self::UV_0).unwrap()[index]
            }
            None => self.attribute::<[[f32; 2]]>(Self::UV_0).unwrap()[face * 3 + vert],
        }
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute_mut::<[[f32; 4]]>(Self::TANGENT).unwrap()[index] = tangent;
            }
            None => {
                self.attribute_mut::<[[f32; 4]]>(Self::TANGENT).unwrap()[face * 3 + vert] = tangent
            }
        }
    }
}

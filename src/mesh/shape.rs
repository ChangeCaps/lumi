use super::Mesh;

pub fn cube(width: f32, height: f32, depth: f32) -> Mesh {
    let mut mesh = Mesh::new();

    let width = width / 2.0;
    let height = height / 2.0;
    let depth = depth / 2.0;

    let positions = vec![
        // Front
        [-width, -height, depth],
        [width, -height, depth],
        [width, height, depth],
        [-width, height, depth],
        // Back
        [-width, -height, -depth],
        [-width, height, -depth],
        [width, height, -depth],
        [width, -height, -depth],
        // Left
        [-width, -height, depth],
        [-width, height, depth],
        [-width, height, -depth],
        [-width, -height, -depth],
        // Right
        [width, -height, -depth],
        [width, height, -depth],
        [width, height, depth],
        [width, -height, depth],
        // Top
        [-width, height, depth],
        [width, height, depth],
        [width, height, -depth],
        [-width, height, -depth],
        // Bottom
        [-width, -height, depth],
        [-width, -height, -depth],
        [width, -height, -depth],
        [width, -height, depth],
    ];

    let normals = vec![
        // Front
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Back
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // Left
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Right
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Top
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Bottom
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];

    let uvs = vec![
        // Front
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        // Back
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        // Left
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        // Right
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        // Top
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        // Bottom
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Left
        12, 13, 14, 14, 15, 12, // Right
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20, // Bottom
    ];

    mesh.insert_attribute(Mesh::POSITION, positions);
    mesh.insert_attribute(Mesh::NORMAL, normals);
    mesh.insert_attribute(Mesh::UV_0, uvs);

    mesh.insert_indices(indices);

    mesh
}

pub fn uv_sphere(radius: f32, segments: u32) -> Mesh {
    let mut mesh = Mesh::new();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut index = 0;

    for y in 0..=segments {
        let v = y as f32 / segments as f32;
        let latitude = (v * std::f32::consts::PI) - std::f32::consts::FRAC_PI_2;

        for x in 0..=segments {
            let u = x as f32 / segments as f32;
            let longitude = u * std::f32::consts::PI * 2.0;

            let normal = [
                longitude.cos() * latitude.cos(),
                latitude.sin(),
                longitude.sin() * latitude.cos(),
            ];
            let position = [normal[0] * radius, normal[1] * radius, normal[2] * radius];

            positions.push(position);
            normals.push(normal);
            uvs.push([u, v]);

            if x > 0 && y > 0 {
                let a = index - segments as usize - 2;
                let b = index - segments as usize - 1;
                let c = index - 1;
                let d = index;

                indices.push(a as u32);
                indices.push(c as u32);
                indices.push(b as u32);

                indices.push(b as u32);
                indices.push(c as u32);
                indices.push(d as u32);
            }

            index += 1;
        }
    }

    mesh.insert_attribute(Mesh::POSITION, positions);
    mesh.insert_attribute(Mesh::NORMAL, normals);
    mesh.insert_attribute(Mesh::UV_0, uvs);
    mesh.insert_indices(indices);

    mesh
}

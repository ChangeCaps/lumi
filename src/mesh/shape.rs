use glam::{Mat3, Vec3};

use super::Mesh;

pub fn arrow_gizmo(direction: Vec3) -> Mesh {
    let up = if direction.y.abs() > 0.9 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let right = direction.cross(up).normalize();
    let up = direction.cross(right).normalize();

    let matrix = Mat3::from_cols(right, direction, up);

    let positions = vec![
        matrix * Vec3::new(-1.0523944e-8, -2.2351742e-8, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, -2.2351742e-8, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, -2.2351742e-8, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, -2.2351742e-8, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, 0.20000002, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, 0.20000002, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, 0.20000002, -0.049999967),
        matrix * Vec3::new(-1.0523944e-8, 0.20000002, -0.049999967),
        matrix * Vec3::new(0.035355322, -2.2351742e-8, -0.035355315),
        matrix * Vec3::new(0.035355322, -2.2351742e-8, -0.035355315),
        matrix * Vec3::new(0.035355322, 0.20000002, -0.035355315),
        matrix * Vec3::new(0.035355322, 0.20000002, -0.035355315),
        matrix * Vec3::new(0.050000004, -2.2351742e-8, 2.6539654e-8),
        matrix * Vec3::new(0.050000004, -2.2351742e-8, 2.6539654e-8),
        matrix * Vec3::new(0.050000004, 0.20000002, 2.6539654e-8),
        matrix * Vec3::new(0.050000004, 0.20000002, 2.6539654e-8),
        matrix * Vec3::new(0.035355322, -2.2351742e-8, 0.035355374),
        matrix * Vec3::new(0.035355322, -2.2351742e-8, 0.035355374),
        matrix * Vec3::new(0.035355322, 0.20000002, 0.035355374),
        matrix * Vec3::new(0.035355322, 0.20000002, 0.035355374),
        matrix * Vec3::new(-1.4895082e-8, -2.2351742e-8, 0.050000027),
        matrix * Vec3::new(-1.4895082e-8, -2.2351742e-8, 0.050000027),
        matrix * Vec3::new(-1.4895082e-8, 0.20000002, 0.050000027),
        matrix * Vec3::new(-1.4895082e-8, 0.20000002, 0.050000027),
        matrix * Vec3::new(-0.035355337, -2.2351742e-8, 0.035355374),
        matrix * Vec3::new(-0.035355337, -2.2351742e-8, 0.035355374),
        matrix * Vec3::new(-0.035355337, 0.20000002, 0.035355374),
        matrix * Vec3::new(-0.035355337, 0.20000002, 0.035355374),
        matrix * Vec3::new(-0.05000002, -2.2351742e-8, 2.375784e-8),
        matrix * Vec3::new(-0.05000002, -2.2351742e-8, 2.375784e-8),
        matrix * Vec3::new(-0.05000002, 0.20000002, 2.375784e-8),
        matrix * Vec3::new(-0.05000002, 0.20000002, 2.375784e-8),
        matrix * Vec3::new(-0.035355367, -2.2351742e-8, -0.035355315),
        matrix * Vec3::new(-0.035355367, -2.2351742e-8, -0.035355315),
        matrix * Vec3::new(-0.035355367, 0.20000002, -0.035355315),
        matrix * Vec3::new(-0.035355367, 0.20000002, -0.035355315),
        matrix * Vec3::new(0.07000354, 0.20000002, -0.070003554),
        matrix * Vec3::new(0.07000354, 0.20000002, -0.070003554),
        matrix * Vec3::new(-9.3579295e-9, 0.20000002, -0.09899996),
        matrix * Vec3::new(-9.3579295e-9, 0.20000002, -0.09899996),
        matrix * Vec3::new(-9.3579295e-9, 0.20000002, -0.09899996),
        matrix * Vec3::new(-9.3579295e-9, 0.20000002, -0.09899996),
        matrix * Vec3::new(0.099000014, 0.20000002, 2.4419778e-8),
        matrix * Vec3::new(0.099000014, 0.20000002, 2.4419778e-8),
        matrix * Vec3::new(0.07000354, 0.20000002, 0.070003614),
        matrix * Vec3::new(0.07000354, 0.20000002, 0.070003614),
        matrix * Vec3::new(-1.801278e-8, 0.20000002, 0.09900002),
        matrix * Vec3::new(-1.801278e-8, 0.20000002, 0.09900002),
        matrix * Vec3::new(-0.070003554, 0.20000002, 0.070003614),
        matrix * Vec3::new(-0.070003554, 0.20000002, 0.070003614),
        matrix * Vec3::new(-0.09900003, 0.20000002, 1.8911788e-8),
        matrix * Vec3::new(-0.09900003, 0.20000002, 1.8911788e-8),
        matrix * Vec3::new(-0.07000362, 0.20000002, -0.070003554),
        matrix * Vec3::new(-0.07000362, 0.20000002, -0.070003554),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.5087426e-8, 0.40000004, 2.896413e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
        matrix * Vec3::new(-1.3038516e-8, -2.2351742e-8, 2.8871e-8),
    ];

    let indices = vec![
        3, 7, 11, 3, 11, 9, 9, 11, 15, 9, 15, 13, 13, 15, 19, 13, 19, 17, 17, 19, 23, 17, 23, 20,
        20, 23, 26, 20, 26, 24, 24, 26, 30, 24, 30, 28, 35, 31, 51, 35, 51, 53, 28, 30, 34, 28, 34,
        32, 32, 34, 6, 32, 6, 2, 12, 16, 67, 40, 52, 54, 14, 10, 36, 14, 36, 42, 27, 22, 46, 27,
        46, 49, 4, 35, 53, 4, 53, 38, 10, 5, 39, 10, 39, 36, 18, 14, 42, 18, 42, 44, 31, 27, 49,
        31, 49, 51, 22, 18, 44, 22, 44, 46, 50, 48, 56, 47, 45, 58, 43, 37, 60, 52, 50, 55, 48, 47,
        57, 45, 43, 59, 37, 41, 61, 25, 29, 64, 16, 21, 66, 29, 33, 63, 8, 12, 68, 21, 25, 65, 33,
        0, 62, 1, 8, 69,
    ];

    let mut mesh = Mesh::new();
    mesh.insert_positions(positions);
    mesh.insert_indices(indices);

    mesh.generate_normals();

    mesh
}

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

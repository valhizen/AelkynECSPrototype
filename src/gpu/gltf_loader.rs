use super::buffer::Vertex;

pub struct LoadModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn load_gltf(path: &str) -> LoadModel {
    let (document, buffers, _images) =
        gltf::import(path).unwrap_or_else(|e| panic!("Failed to load model '{path}': {e}"));

    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();

    // Start from root nodes (scene nodes), walk children recursively
    for scene in document.scenes() {
        for node in scene.nodes() {
            collect_node(
                &node,
                &buffers,
                glam::Mat4::IDENTITY, // root has no parent transform
                &mut all_vertices,
                &mut all_indices,
            );
        }
    }

    println!(
        "Loaded '{}': {} vertices, {} indices",
        path,
        all_vertices.len(),
        all_indices.len()
    );

    LoadModel {
        vertices: all_vertices,
        indices: all_indices,
    }
}

fn collect_node(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    parent_transform: glam::Mat4,
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u32>,
) {
    // This node's world transform = parent * local
    let local = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local;
    let normal_matrix = glam::Mat3::from_mat4(world_transform.inverse().transpose());

    // If this node has a mesh, process it
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));

            let positions: Vec<[f32; 3]> = match reader.read_positions() {
                Some(p) => p.collect(),
                None => continue,
            };

            let vertex_count = positions.len();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; vertex_count]);

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; vertex_count]);

            let colors: Vec<[f32; 3]> = reader
                .read_colors(0)
                .map(|c| c.into_rgb_f32().collect())
                .unwrap_or_else(|| {
                    let base = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_factor();
                    vec![[base[0], base[1], base[2]]; vertex_count]
                });

            let base_vertex = all_vertices.len() as u32;

            for i in 0..vertex_count {
                let pos = glam::Vec3::from(positions[i]);
                let transformed_pos = world_transform.transform_point3(pos);

                let norm = glam::Vec3::from(normals[i]);
                let transformed_norm = (normal_matrix * norm).normalize();

                all_vertices.push(Vertex {
                    position: transformed_pos.into(),
                    normal: transformed_norm.into(),
                    color: colors[i],
                    uv: uvs[i],
                });
            }

            let indices: Vec<u32> = reader
                .read_indices()
                .map(|idx| idx.into_u32().collect())
                .unwrap_or_else(|| (0..vertex_count as u32).collect());

            for index in indices {
                all_indices.push(base_vertex + index);
            }
        }
    }

    // Recurse into children with accumulated transform
    for child in node.children() {
        collect_node(&child, buffers, world_transform, all_vertices, all_indices);
    }
}

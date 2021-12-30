
use cg::{EuclideanVector, Vector3};

use glium::backend::{Facade};
use glium::vertex::{
    Vertex, VertexBuffer,
    BufferCreationError as VertexBufferCreationError
};
use glium::index::{
    IndicesSource, NoIndices, IndexBuffer, PrimitiveType,
    BufferCreationError as IndexBufferCreationError
};

use util::{FixedDimension, CardinalDirection};

use terrain::{Terrain};

#[derive(Copy, Clone)]
pub struct LineVertex {
    pub v_pos: [f32; 3],
    pub v_color: [f32; 3],
}

#[derive(Copy, Clone)]
pub struct FaceVertex {
    pub v_pos: [f32; 3],
    pub v_tex_pos: [f32; 2],
    pub v_normal: [f32; 3],
}

implement_vertex!(LineVertex, v_pos, v_color);
implement_vertex!(FaceVertex, v_pos, v_normal, v_tex_pos);

pub struct Mesh<V> {
    pub verts: Vec<V>,
    pub inds: Option<Vec<u32>>,
    pub primitive_type: PrimitiveType,
}

pub struct UploadedMesh<V: Copy + Vertex> {
    pub vbo: VertexBuffer<V>,
    pub ibo: UploadedIndices,
}

pub enum UploadedIndices {
    NoIndices(NoIndices),
    IndexBuffer(IndexBuffer<u32>),
}

#[derive(Debug)]
pub enum MeshUploadError {
    IndexBufferCreationError(IndexBufferCreationError),
    VertexBufferCreationError(VertexBufferCreationError),
}

impl<V: Copy + Vertex> Mesh<V> {
    pub fn upload<F: Facade>(&self, facade: &F) -> Result<UploadedMesh<V>, MeshUploadError> {
        let vbo_result = VertexBuffer::new(facade, &self.verts);

        let ibo_result = match self.inds {
            None => Ok(NoIndices(self.primitive_type).into()),
            Some(ref inds) => {
                IndexBuffer::new(facade, self.primitive_type, inds)
                    .map(|buf| buf.into())
            },
        };

        vbo_result
            .map_err(|err| err.into())
            .and_then(|vbo| {
                ibo_result
                    .map_err(|err| err.into())
                    .map(|ibo| (vbo, ibo))
            })
            .map(|(vbo, ibo)| {
                UploadedMesh {
                    vbo: vbo,
                    ibo: ibo,
                }
            })
    }
}

pub fn show_normals(mesh: &Mesh<FaceVertex>, l: f32, colors: [[f32; 3]; 2]) -> Mesh<LineVertex> {
    let verts = mesh.verts.iter()
        .flat_map(|v| {
            vec![
                LineVertex {
                    v_pos: v.v_pos,
                    v_color: colors[0],
                },
                LineVertex {
                    v_pos: [
                        v.v_pos[0] + v.v_normal[0] * l,
                        v.v_pos[1] + v.v_normal[1] * l,
                        v.v_pos[2] + v.v_normal[2] * l,
                    ],
                    v_color: colors[1],
                }
            ]
        })
        .collect();

    Mesh {
        verts: verts,
        inds: None,
        primitive_type: PrimitiveType::LinesList,
    }
}

pub fn terrain_mesh(terrain: &Terrain, sample_size: [f32; 2], samples_per_tex: usize)
        -> Mesh<FaceVertex> {

    let width_z = terrain.fixed_dim.height();
    let width_x = terrain.vec.len() / width_z;

    let normals = terrain.vec.iter().zip(terrain.fixed_dim.coords_iter())
        .map(|(&height, coords)| {
            use util::CardinalDirection as CD;

            let coords = [coords[0] as i32, coords[1] as i32];

            let rel_coords = {
                let mut rel_coords = [None; 4];
                rel_coords[CD::Up.index()] = Some([coords[0], coords[1] + 1]);

                rel_coords[CD::Down.index()] = Some([coords[0], coords[1] - 1]);
                rel_coords[CD::Left.index()] = Some([coords[0] - 1, coords[1]]);
                rel_coords[CD::Right.index()] = Some([coords[0] + 1, coords[1]]);
                [
                    rel_coords[0].unwrap(),
                    rel_coords[1].unwrap(),
                    rel_coords[2].unwrap(),
                    rel_coords[3].unwrap(),
                ]
            };

            let get_h = |dir: CD| {
                let alt = || {
                    let inv_index = dir.inv().index();
                    let inv_dir_coords = rel_coords[inv_index];
                    let inv_dir_coords = [inv_dir_coords[0] as usize, inv_dir_coords[1] as usize];
                    let inv_height = *terrain.get(inv_dir_coords).unwrap_or(&height); // dunno rly
                    2.0 * height - inv_height
                };

                let alt = || height;

                let dir_coords = rel_coords[dir.index()];
                if dir_coords[0] >= 0 && dir_coords[1] >= 0 && dir_coords[0] < width_x as i32 && dir_coords[1] < width_z as i32 {
                    let dir_coords = [dir_coords[0] as usize, dir_coords[1] as usize];
                    terrain.get(dir_coords)
                        .map(|x| *x)
                        .unwrap_or(alt())
                } else {
                    alt()
                }
            };

            terrain_normal(get_h(CD::Right), get_h(CD::Left), get_h(CD::Up), get_h(CD::Down))
        })
        .collect::<Vec<[f32; 3]>>();

    let verts = terrain.vec.iter().enumerate()
        .map(|(i, &height)| (terrain.fixed_dim.to_coords(i), height))
        .filter(|&(coords, _)| {
            // heights on the edge don't produce faces
            coords[0] != width_x - 1 && coords[1] != width_z - 1
        })
        // each coords creates the faces between itself and the right and up coords
        .flat_map(|(coords, height)| {
            let x = coords[0] as f32 * sample_size[0];
            let z = coords[1] as f32 * sample_size[1];

            let up_coords = [coords[0], coords[1] + 1];
            let right_coords = [coords[0] + 1, coords[1]];
            let oppo_coords = [coords[0] + 1, coords[1] + 1];

            let up_x = up_coords[0] as f32 * sample_size[0];
            let up_z = up_coords[1] as f32 * sample_size[1];
            let right_x = right_coords[0] as f32 * sample_size[0];
            let right_z = right_coords[1] as f32 * sample_size[1];
            let oppo_x = oppo_coords[0] as f32 * sample_size[0];
            let oppo_z = oppo_coords[1] as f32 * sample_size[1];

            let to_index = |c| terrain.fixed_dim.to_index(c).unwrap();
            let oppo_height = terrain.vec[to_index(oppo_coords)];
            let up_height = terrain.vec[to_index(up_coords)];
            let right_height = terrain.vec[to_index(right_coords)];
            // let down_height = if coords[1] == 0 {
            //     height + (height - up_height)
            // } else {
            //     terrain.vec[to_index([coords[0], coords[1] - 1])]
            // };
            // let left_height = if coords[0] == 0 {
            //     height + (height - right_height)
            // } else {
            //     terrain.vec[to_index([coords[0] - 1, coords[1]])]
            // };

            let normal = normals[terrain.fixed_dim.to_index(coords).unwrap()];
            let _right_normal = normals[terrain.fixed_dim.to_index(right_coords).unwrap()];
            let _up_normal = normals[terrain.fixed_dim.to_index(up_coords).unwrap()];
            let oppo_normal = normals[terrain.fixed_dim.to_index(oppo_coords).unwrap()];

            let part_count = samples_per_tex;
            let x_part = (coords[0] % part_count) as f32;
            let y_part = (coords[1] % part_count) as f32;
            let part_count = part_count as f32;
            let tex_coords00 = [
                x_part * (1.0 / part_count),
                y_part * (1.0 / part_count),
            ];
            let tex_coords10 = [
                tex_coords00[0] + (1.0 / part_count),
                tex_coords00[1]
            ];
            let tex_coords01 = [
                tex_coords00[0],
                tex_coords00[1] + (1.0 / part_count)
            ];
            let tex_coords11 = [
                tex_coords00[0] + (1.0 / part_count),
                tex_coords00[1] + (1.0 / part_count)
            ];



            let vertex = FaceVertex {
                v_pos: [x, height, z],
                v_tex_pos: tex_coords00,
                v_normal: normal,
            };
            let oppo_vertex = FaceVertex {
                v_pos: [oppo_x, oppo_height, oppo_z],
                v_tex_pos: tex_coords11,
                v_normal: oppo_normal,
            };
            let right_vertex1 = FaceVertex {
                v_pos: [right_x, right_height, right_z],
                v_tex_pos: tex_coords10,
                v_normal: normal,
            };
            let right_vertex2 = FaceVertex {
                v_pos: [right_x, right_height, right_z],
                v_tex_pos: tex_coords10,
                v_normal: oppo_normal,
            };
            let up_vertex1 = FaceVertex {
                v_pos: [up_x, up_height, up_z],
                v_tex_pos: tex_coords01,
                v_normal: normal,
            };
            let up_vertex2 = FaceVertex {
                v_pos: [up_x, up_height, up_z],
                v_tex_pos: tex_coords01,
                v_normal: oppo_normal,
            };

            vec![
                vertex, right_vertex1, up_vertex1,
                oppo_vertex, right_vertex2, up_vertex2,
            ]
        })
        .collect();

    Mesh {
        verts: verts,
        inds: None,
        primitive_type: PrimitiveType::TrianglesList,
    }
}

pub fn tri_normal(verts: [[f32; 3]; 3]) -> [f32; 3] {
    let a = Vector3::new(verts[0][0], verts[0][1], verts[0][2]);
    let b = Vector3::new(verts[1][0], verts[1][1], verts[1][2]);
    let c = Vector3::new(verts[2][0], verts[2][1], verts[2][2]);
    let r = (b - a).cross(c - a).normalize();
    [r.x, r.y, r.z]
}

/// taking four heights from up, down, left, right and returns the normal vector
pub fn terrain_normal(right: f32, left: f32, up: f32, down: f32) -> [f32; 3] {
    let v = Vector3::new(left - right, 2.0, down - up).normalize();
    [v.x, v.y, v.z]
}

impl Into<UploadedIndices> for NoIndices {
    fn into(self) -> UploadedIndices {
        UploadedIndices::NoIndices(self)
    }
}

impl Into<UploadedIndices> for IndexBuffer<u32> {
    fn into(self) -> UploadedIndices {
        UploadedIndices::IndexBuffer(self)
    }
}

impl Into<MeshUploadError> for IndexBufferCreationError {
    fn into(self) -> MeshUploadError {
        MeshUploadError::IndexBufferCreationError(self)
    }
}

impl Into<MeshUploadError> for VertexBufferCreationError {
    fn into(self) -> MeshUploadError {
        MeshUploadError::VertexBufferCreationError(self)
    }
}

impl<'a> Into<IndicesSource<'a>> for &'a UploadedIndices {
    fn into(self) -> IndicesSource<'a> {
        match *self {
            UploadedIndices::NoIndices(x) => x.into(),
            UploadedIndices::IndexBuffer(ref x) => x.into(),
        }
    }
}

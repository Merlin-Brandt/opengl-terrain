
extern crate image;

use std::fs::File;

use glium;
use glium::Surface;
use glium::backend::{Facade};
use glium::texture::{Texture2d, RawImage2d};
use glium::program::{Program};

use util::{NonZero, EnsureNotZero, MappableArray};
use terrain::{self, Area};
use mesh::{self, UploadedMesh, FaceVertex, LineVertex};

pub struct Renderer {
    /// Y-Major
    terrain: UploadedMesh<FaceVertex>,
    normals: UploadedMesh<LineVertex>,
    terrain_tex: Texture2d,
    line_shader: Program,
    face_shader: Program,
}

impl Renderer {
    pub fn new<F: Facade>(facade: &F) -> Result<Renderer, mesh::MeshUploadError> {
        let area = Area {x: 0.0, y: 0.0, w: 1000.0, h: 1000.0};
        let samples = [100.ensure_not_zero(); 2];
        let terrain = terrain::gen_terrain(samples, 12, area, 30.0);
        let samples = samples.map().with(|x| x.val());

        let sample_size = [100.0 / samples[0] as f32, 100.0 / samples[1] as f32];
        let terrain_mesh = mesh::terrain_mesh(&terrain, sample_size, 30);
        let uploaded_terrain = terrain_mesh.upload(facade);

        let terrain_texture = {
            let file = File::open("res/terrain.png").expect("Error opening file");
            let image = image::load(file, image::PNG).expect("Error decoding image").to_rgba();
            let dims = image.dimensions();
            let image = RawImage2d::from_raw_rgba_reversed(image.into_raw(), dims);
            Texture2d::new(facade, image).expect("Error uploading texture")
        };

        let normal_lines_colors = [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]];
        let shown_normals = mesh::show_normals(&terrain_mesh, 0.2, normal_lines_colors).upload(facade);

        uploaded_terrain
            .and_then(|uploaded_terrain| {
                shown_normals.map(|sn| (sn, uploaded_terrain))
            })
            .map(|(shown_normals, terrain)| {
                Renderer {
                    terrain: terrain,
                    normals: shown_normals,
                    terrain_tex: terrain_texture,
                    line_shader: program!(facade,
                        330 => {
                            vertex: r#"
                                #version 330

                                in vec3 v_pos;
                                in vec3 v_color;
                                out vec3 p_color;

                                uniform mat4 projview;
                                uniform mat4 model;

                                void main()
                                {
                                    gl_Position = projview * model * vec4(v_pos, 1.0);
                                    p_color = v_color;
                                }
                            "#,
                            fragment: r#"
                                #version 330

                                in vec3 p_color;
                                out vec4 f_color;

                                void main()
                                {
                                    f_color = vec4(p_color, 1.0);
                                }
                            "#,
                        }).expect("Error creating program"),
                    face_shader: program!(facade,
                        330 => {
                            vertex: r#"
                                #version 330

                                in vec3 v_pos;
                                in vec2 v_tex_pos;
                                in vec3 v_normal;

                                out vec2 p_tex_pos;
                                out vec3 p_normal;

                                uniform mat4 projview;
                                uniform mat4 model;

                                void main()
                                {
                                    gl_Position = projview * model * vec4(v_pos, 1.0);
                                    p_tex_pos = v_tex_pos;
                                    p_normal = v_normal;
                                }
                            "#,
                            fragment: r#"
                                #version 330

                                in vec2 p_tex_pos;
                                in vec3 p_normal;

                                out vec4 f_color;

                                uniform vec3 light_dir;
                                uniform sampler2D tex;

                                void main()
                                {
                                    f_color = texture(tex, p_tex_pos) * dot(p_normal, light_dir);
                                }
                            "#,
                        }).expect("Error creating program"),
                }
            })
    }

    pub fn render<S: Surface>(&mut self, target: &mut S, projview: &[[f32; 4]; 4], _time: f32) {
        let uniforms = uniform! {
            projview: *projview,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0_f32],
            ],
            light_dir: [
                0.3,
                0.4,
                0.1f32,
            ],
            tex: &self.terrain_tex,
        };

        let draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&self.terrain.vbo, &self.terrain.ibo, &self.face_shader, &uniforms, &draw_params).expect("Error drawing");
        if false {
            target.draw(&self.normals.vbo, &self.normals.ibo, &self.line_shader, &uniforms, &draw_params).expect("Error drawing");
        }
    }
}

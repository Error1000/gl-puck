use gl::types::*;
use gl_puck::camera::Camera3D;
use gl_puck::input::KeyboardHandler;
use gl_puck::model::{Model, World3D};
use gl_puck::obj::*;
use gl_puck::{mesh, model};
use gl_wrapper::render::{program, shader, texture};
use gl_wrapper::util::{buffer_obj, aggregator_obj};
use glam::{Mat4, Vec2, Vec3};
use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::run_return::EventLoopExtRunReturn;
use glutin::window::WindowBuilder;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::num::ParseFloatError;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use std::{fs, io};

fn normalise_and_center(pos: &mut Vec<f32>) -> &mut Vec<f32> {
    let mut ma: f32 = 0.0;
    let mut mi: f32 = pos[0];
    // Assumes ma != mi
    for e in pos.iter() {
        if *e > ma {
            ma = *e;
        }
        if *e < mi {
            mi = *e;
        }
    }
    let b: f32 = ma - mi;
    pos.iter_mut().for_each(|e| *e = (*e - mi) / b - 0.5); // Now everything is between -0.5 and 0.5
    pos
}

fn invert_tex(tex: &mut Vec<f32>) -> &mut Vec<f32> {
    let mut a = false;
    tex.iter_mut().for_each(|e| {
        if a {
            *e = 1.0 - *e
        }
        a = !a;
    });
    tex
}

use std::hash::Hash;
#[derive(Clone, Copy, Default)]
struct F32Wrapper(pub f32);

impl F32Wrapper{
    fn key(&self) -> u32{
       self.0.to_bits()
    }
}

impl PartialEq for F32Wrapper{
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for F32Wrapper{}
impl Hash for F32Wrapper{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl FromStr for F32Wrapper{
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        f32::from_str(s).map(|v| Self(v))
    }
}

impl From<f32> for F32Wrapper{
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<F32Wrapper> for f32{
    fn from(v: F32Wrapper) -> Self{
        v.0
    }
}


// TODO: Add better input handling( simpler than glutin )( look at create gilrs for controller input maybe ), maybe obscure the glutin event loop a bit more, figure out why movement is jittery even though i'm adapting speed to deltaT in-between frames ( possibly not my fault and it's just that the timer might be inaccurate but it seems way too off for that )
fn main() -> io::Result<()> {
    let mut prog_bouncer = program::ProgramBouncer::new();
    let mut vao_bouncer  = aggregator_obj::VAOBouncer::new();
    let mut vbo_bouncer  = buffer_obj::VBOBouncer::new();
    let mut ibo_bouncer  = buffer_obj::IBOBouncer::new();

    //RESOURCES
    let fov: f32 = 70.0_f32.to_radians();
    const Z_NEAR: f32 = 0.001;
    let mut w_width: u32 = 400;
    let mut w_height: u32 = 400;
    const OBJ_FILE: &str = "dragon.obj";
    const FRAGMENT_SHADER_FILE: &str = "fragmentShader.glsl";
    const VERTEX_SHADER_FILE: &str = "vertexShader.glsl";
    const TEXTURE_FILE: &str = "rungholt-RGBA.png";
    const MOUSE_SENSITIVITY: f32 = 13.0;

    let mut events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: w_width,
            height: w_height,
        })
        .with_visible(false);

    let gl_window = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window, &events_loop)
        .expect("Failed to create window!");
    let gl_window = gl_wrapper::init(gl_window).expect("Failed to create opengl context");

    let mut proj = Mat4::perspective_infinite_lh(fov, (w_width as f32) / (w_height as f32), Z_NEAR);

    let mut cam = Camera3D::new();

    println!("Loading obj ...");
    let (pos_vbo, tex_vbo, ind_ibo) = {
        let mut o = ObjData::<F32Wrapper, u32>::new(Dimension::ThreeDim, Some(Dimension::TwoDim), None);
        {
            let t1 = Instant::now();
            o.load(&mut File::open(OBJ_FILE)?)?;
            println!(
                "Took {} seconds to load obj file!",
                t1.elapsed().as_secs_f32()
            );
        }
        let mut actual_pos_data: Vec<f32> =  o.pos_data.get_vals().iter().map(|v|f32::from(*v)).collect();
        let mut actual_tex_data: Vec<f32> =  o.tex_data.expect("Texture mapping data should be there since we requested it.").get_vals().iter().map(|v|f32::from(*v)).collect();
        (
            buffer_obj::VBO::<GLfloat>::with_data(
                &mut vbo_bouncer,
                &[3],
                normalise_and_center(&mut actual_pos_data).as_slice(),
                gl::STATIC_DRAW,
            )
            .expect("Failed to create pos_vbo!"),
            buffer_obj::VBO::<GLfloat>::with_data(
                &mut vbo_bouncer,
                &[2],
                invert_tex(&mut actual_tex_data).as_slice(),
                gl::STATIC_DRAW,
            )
            .expect("Failed to create tex_vbo!"),
            buffer_obj::IBO::<GLuint>::with_data(&mut ibo_bouncer, &o.indicies.as_slice(), gl::STATIC_DRAW)
                .expect("Failed to create ind_ibo!"),
        )
    };
    println!("Done!");

    let mut program = {
        let vs_source = fs::read_to_string(VERTEX_SHADER_FILE)?;
        let fs_source = fs::read_to_string(FRAGMENT_SHADER_FILE)?;

        let vs = shader::VertexShader::new(vs_source.as_ref()).unwrap();
        let fs = shader::FragmentShader::new(fs_source.as_ref()).unwrap();
        program::Program::new(&[&vs.into(), &fs.into()]).unwrap()
    };
    let mut program = program.bind_mut(&mut prog_bouncer);
    program.load_attribute("position").expect("Load attribute 'position'");
    program.load_attribute("tex_coord").expect("Load attribute 'tex_coord'");
    program.load_uniform("mvp").expect("Load uniform 'mvp'");
    program.load_sampler("obj_tex").expect("Load sampler 'obj_tex'");

    {let id = program.get_sampler_id("obj_tex").unwrap().try_into().unwrap(); program.set_uniform_i32(id, 0);}
    let mut tex_bouncer = texture::TextureBouncer::<0>::new();

    let mut t = {
        let im = image::open(&Path::new(TEXTURE_FILE))
            .expect("Failed to load image!")
            .into_rgba8();
        texture::Texture2D::with_data(
                &mut tex_bouncer,
        [
                im.width().try_into().unwrap(),
                im.height().try_into().unwrap(),
            ],
            im.as_ref(),
            gl::RGBA,
        )
        .expect("Failed to create texture")
    };
    let mut t = t.bind_mut(&mut tex_bouncer);
    t.set_mag_filter_of_bound_tex(gl::NEAREST);
    t.set_min_filter_of_bound_tex(gl::NEAREST);

    let mut model = {
        let m = mesh::UnboundMesh::new(&ind_ibo);
        model::UnboundModel3D::new(m)
    };
    let mut model = model.bind(&mut vao_bouncer, &mut ibo_bouncer);
    // Prepare model for use with program
    let pos_vbo = pos_vbo.bind(&mut vbo_bouncer);
    model
        .adapt_model_to_attrib(
            &pos_vbo,
            program
                .get_attribute_id("position")
                .expect("Attribute not loaded!"),
        )
        .unwrap();
    let tex_vbo = tex_vbo.bind(&mut vbo_bouncer);
    model
        .adapt_model_to_attrib(
            &tex_vbo,
            program
                .get_attribute_id("tex_coord")
                .expect("Attribute not loaded!"),
        )
        .unwrap();
    model.adapt_model_to_program(&program).unwrap();

    println!("Showing window!");
    gl_window.window().set_visible(true);

    let mut keyb = KeyboardHandler::new();

    let mut start = Instant::now();
    gl_wrapper::set_gl_clear_color(178.0 / 255.0, 1.0, 1.0, 0.0).expect("Setting clear color!");
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
    let mut last_mouse_change: (f32, f32) = (0.0, 0.0);
    let mut in_control = false;
    events_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Window stuff
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(PhysicalSize { width, height }) => {
                        gl_wrapper::set_gl_draw_size(width, height).unwrap();

                        w_width = width;
                        w_height = height;
                        // Remake projection matrix
                        proj = Mat4::perspective_infinite_lh(
                            fov,
                            (w_width as f32) / (w_height as f32),
                            Z_NEAR,
                        );
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input: i, .. } => {
                        keyb.handle(&i);
                    }
                    WindowEvent::CursorEntered { .. } => {
                            gl_window.window().set_cursor_grab(true).unwrap();
                            gl_window.window().set_cursor_visible(false);
                            in_control = true;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent { event, .. } => {
                if let glutin::event::DeviceEvent::MouseMotion { delta } = event {
                    last_mouse_change.0 += delta.0 as f32;
                    last_mouse_change.1 += delta.1 as f32;
                }
            }

            Event::RedrawEventsCleared => {
                if keyb
                    .is_pressed(glutin::event::VirtualKeyCode::Escape)
                    .unwrap_or(false)
                {
                    gl_window.window().set_cursor_grab(false).unwrap();
                    gl_window.window().set_cursor_visible(true);
                    in_control = false;
                }

                let delta_t = start.elapsed().as_secs_f32();
                let fps = 1.0 / delta_t;

                if fps <= 60.0 {
                    // If fps is too high ( yes this could be a problem as it makes the movement of the object extremely small and basically introduces a lag spike ( or better named a performance spike ) which is bad) just waste the current cycle and do not reset timer to let it accumulate time
                    println!("{}", fps);
                    if in_control {
                        cam.rotate(
                            Vec3::new(
                                -last_mouse_change.1 * MOUSE_SENSITIVITY,
                                -last_mouse_change.0 * MOUSE_SENSITIVITY,
                                0.0,
                            ) * delta_t,
                        );
                        let speed = 0.01;
                        if keyb
                            .is_pressed(glutin::event::VirtualKeyCode::W)
                            .unwrap_or(false)
                        {
                            cam.masked_step(Vec2::new(0.0, speed) * delta_t, Vec3::new(1.0, 0.0, 1.0));
                        } else if keyb
                            .is_pressed(glutin::event::VirtualKeyCode::S)
                            .unwrap_or(false)
                        {
                            cam.masked_step(Vec2::new(0.0, -speed) * delta_t, Vec3::new(1.0, 0.0, 1.0));
                        }
                        if keyb
                            .is_pressed(glutin::event::VirtualKeyCode::A)
                            .unwrap_or(false)
                        {
                            cam.step(Vec2::new(-speed, 0.0) * delta_t);
                        } else if keyb
                            .is_pressed(glutin::event::VirtualKeyCode::D)
                            .unwrap_or(false)
                        {
                            cam.step(Vec2::new(speed, 0.0) * delta_t);
                        }

                        if keyb
                            .is_pressed(glutin::event::VirtualKeyCode::Space)
                            .unwrap_or(false)
                        {
                            cam.strafe(Vec3::new(0.0, speed, 0.0) * delta_t);
                        } else if keyb
                            .is_pressed(glutin::event::VirtualKeyCode::LShift)
                            .unwrap_or(false)
                        {
                            cam.strafe(Vec3::new(0.0, -speed, 0.0) * delta_t);
                        }
                    }

                    last_mouse_change = (0.0, 0.0);
                    {
                        let model_mat = *model.get_mat();
                        let id = program.get_uniform_id("mvp").expect("Uniform not loaded!");
                        program.set_uniform_mat4_f32(
                            i32::try_from(id).unwrap(),
                            (proj * *cam.get_mat() * model_mat).as_ref(),
                        );
                    }
                    model.render(&program).unwrap();
                    gl_window.swap_buffers().unwrap();
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    }
                    start = Instant::now();
                }
            }
            _ => {}
        }
    });

    Ok(())
}

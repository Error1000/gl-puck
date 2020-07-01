extern crate gl_puck;
extern crate gl_wrapper;
extern crate glutin;
extern crate image;

use gl_puck::model;
use gl_puck::model::Renderable;
use gl_puck::model::World2D;
use gl_puck::{input, mesh};
use gl_wrapper::render::texture::TextureFunc;
use gl_wrapper::render::*;
use gl_wrapper::util::*;

use glutin::dpi::PhysicalSize;
use std::convert::{TryFrom, TryInto};

use gl::types::*;
use std::str;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

use std::path::Path;

use gl_puck::camera::Camera2D;
use glam::{Mat3, Vec2};
use glutin::platform::desktop::EventLoopExtDesktop;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

// Vertex data
static VERTEX_DATA: [GLfloat; 8] = [-1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0];

// Tex data
static TEX_DATA: [GLfloat; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];

const TEST_ZOOM_OUT: GLfloat = 101.0;

// Tex2 data
static TEX2_DATA: [GLfloat; 8] = [
    0.0,
    0.0,
    1.0 * TEST_ZOOM_OUT,
    0.0,
    1.0 * TEST_ZOOM_OUT,
    1.0 * TEST_ZOOM_OUT,
    0.0,
    1.0 * TEST_ZOOM_OUT,
];

// Indices data
static IND_DATA: [GLushort; 6] = [0, 1, 3, 1, 2, 3];

// TODO List: Add mesh algorithms ( mesh simplification, .. ), add view frustum culling for 2d and 3d
fn main() {
    let mut window_width: f32 = 400.0;
    let mut window_height: f32 = 400.0;
    let mut events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: window_width,
            height: window_height,
        })
        .with_visible(false); // Hide window while loading to make it less annoying

    let gl_window = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window, &events_loop)
        .expect("Failed to create window!");

    // Load the OpenGL function pointers
    let gl_window = gl_wrapper::init(gl_window).expect("Failed to acquire gl context!");

    let mut proj = gl_puck::make_ortho_2d(window_width, window_height);

    let mut cam = Camera2D::new();

    println!("Window created but hidden!");
    println!("OpenGL Version: {}", gl_wrapper::get_gl_version_str());

    // Shader sources
    static VS_SRC: &str = "
#version 150
attribute vec2 position;
attribute vec2 tex_coord;
out vec2 pass_tex_coord;
uniform mat3 mvp;
void main() {
    pass_tex_coord = tex_coord;
    gl_Position = vec4( mvp * vec3(position, 1.0), 1.0);
}";

    static FS_SRC: &str = "
#version 150
out vec4 out_color;
uniform sampler2D obj_tex;
in vec2 pass_tex_coord;
void main() {
    out_color = texture2D(obj_tex, pass_tex_coord);
}";

    // Create GLSL shaders
    println!("Loading shaders ...");
    let program = RefCell::new({
        // Program and shader provide their own error messages
        let vs = shader::VertexShader::new(VS_SRC).unwrap();
        let fs = shader::FragmentShader::new(FS_SRC).unwrap();
        program::Program::new(&[&vs.into(), &fs.into()]).unwrap()
    });

    program.borrow().bind_program();
    program
        .borrow_mut()
        .load_attribute("position".to_owned())
        .expect("Failed to load data from shader!");
    program
        .borrow_mut()
        .load_attribute("tex_coord".to_owned())
        .expect("Failed to load data from shader!");
    program
        .borrow_mut()
        .load_uniform("mvp".to_owned())
        .expect("Failed to load data form shader!");
    program
        .borrow_mut()
        .load_sampler("obj_tex".to_owned())
        .expect("Failed to load data from shader!");
    println!("Done!");

    // Load textures
    println!("Loading textures ...");
    let t = {
        let im = image::open(&Path::new("apple.png"))
            .expect("Failed to load texture! Are you sure it exists?")
            .into_rgba();
        texture::Texture2D::with_data(
            [
                im.width().try_into().unwrap(),
                im.height().try_into().unwrap(),
            ],
            im.as_ref(),
            gl::RGBA,
        )
        .expect("Failed to create texture!")
    };

    let mut tile = {
        let im = image::open(&Path::new("t.jpg"))
            .expect("Failed to load texture! Are you sure it exists?")
            .into_rgba();
        texture::Texture2D::with_data(
            [
                im.width().try_into().unwrap(),
                im.height().try_into().unwrap(),
            ],
            im.as_ref(),
            gl::RGBA,
        )
        .expect("Failed to create texture!")
    };
    tile.set_x_wrap_of_bound_tex(gl::REPEAT.try_into().unwrap());
    tile.set_y_wrap_of_bound_tex(gl::REPEAT.try_into().unwrap());

    println!("Done!");

    // Load mesh data ( indices, vertices, uv data )
    println!("Loading mesh ...");
    // NOTE: Creating a vbo with data auto binds it, creating a vbo using new does not
    let pos_vbo = Rc::new(
        buffer_obj::VBO::<GLfloat>::with_data(&[2], &VERTEX_DATA, gl::STATIC_DRAW)
            .expect("Failed to upload data to vbo!"),
    );
    let tex_vbo = Rc::new(
        buffer_obj::VBO::<GLfloat>::with_data(&[2], &TEX_DATA, gl::STATIC_DRAW)
            .expect("Failed to upload data to vbo!"),
    );
    let ind_ibo = Rc::new(
        buffer_obj::IBO::<GLushort>::with_data(&IND_DATA, gl::STATIC_DRAW)
            .expect("Failed to upload data to ibo!"),
    );

    let mut apple = {
        let m = mesh::Mesh::new(
            &[
                (
                    &pos_vbo,
                    program.borrow().get_attribute_id("position").unwrap(),
                ),
                (
                    &tex_vbo,
                    program.borrow().get_attribute_id("tex_coord").unwrap(),
                ),
            ],
            &ind_ibo,
        )
        .unwrap();
        model::Model2D::new(m, &program)
    };

    let tex2_vbo = buffer_obj::VBO::<GLfloat>::with_data(&[2], &TEX2_DATA, gl::STATIC_DRAW)
        .expect("Failed to upload to vbo!");

    let mut test = {
        let m = mesh::Mesh::new(
            &[
                (
                    &pos_vbo,
                    program.borrow().get_attribute_id("position").unwrap(),
                ),
                (
                    &tex2_vbo,
                    program.borrow().get_attribute_id("tex_coord").unwrap(),
                ),
            ],
            &ind_ibo,
        )
        .unwrap();
        model::Model2D::new(m, &program)
    };

    // We need to specify it in half due to projection ( thankfully tho speeds are 1:1 )
    // TODO: Fix this
    apple.set_size(Vec2::new(400.0 / 2.0, 400.0 / 2.0));
    test.set_size(apple.get_size().clone() * TEST_ZOOM_OUT); // make sure tile is the same size as apple
    println!("Done!");

    println!("Showing window!");
    gl_window.window().set_visible(true);
    // NOTE: Slight hack to make sure inputs are responsive-ish( was having problems with specifying stuff inside the key down/ key up events ), should fix later, priority medium to low
    let mut keyb = input::KeyboardHandler::new();
    let mut start = Instant::now();
    gl_wrapper::set_gl_clear_color(0.0, 0.0, 1.0, 1.0);

    const SPEED: f32 = 400.0; // pixels/second
    events_loop.run_return( |event, _, control_flow| {
        // Unless we re write the control flow just wait until another event arrives when this iteration finished
        *control_flow = ControlFlow::Poll;
        match event {
            // Window stuff
            Event::WindowEvent { event, .. } => {
                match event{
                    WindowEvent::Resized(PhysicalSize{width, height}) => {
                        gl_wrapper::set_gl_draw_size(width, height).unwrap();

                        window_width = width as f32;
                        window_height = height as f32;
                        // Remake projection matrix
                        proj = gl_puck::make_ortho_2d(window_width, window_height);
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input: i, .. } => keyb.handle(&i),

                    /* WindowEvent::CursorMoved{position: p, ..} => {
                        let p_x = p.x as f32;
                        let p_y = p.y as f32;
                        // TODO: Fix coordinate system ( maybe look at using mat4 instead of mat3 in 2d )
                        println!("X: {}, Y: {}", p_x-window_width/2.0, -p_y+window_height/2.0);
                        apple.set_pos(Vec2::new(p_x-window_width/2.0, -p_y+window_height/2.0));
                    }, */
                    _ => {}
                }
            },

            Event::RedrawEventsCleared => {
                // TODO: Eyeballing the output of this it seems like the app still has some microstutters, but they don't really occur usually plus aren't that annoying ans could probably be resolved by averaging the elapsed time over multiple frames to get a smoother movement and probably resolve the stutters, whatever
                let delta_t = start.elapsed().as_secs_f32();
                let fps = 1.0 / delta_t;

                if fps <= 60.0 { // If fps is too high ( yes this could be a problem as it makes the movement of the object extremely small and basically introduces a lag spike ( or bette named a performance spike ) which is bad) just waste the current cycle and do not reset timer to let it accumulate time
                    // speed of 200 u.m. ( units of measurement ) ( in this case pixels ) / sec.
                    if keyb.is_pressed(glutin::event::VirtualKeyCode::W).unwrap_or(false) {
                        apple.strafe(Vec2::new(0.0, SPEED * delta_t));
                    }
                    if  keyb.is_pressed(glutin::event::VirtualKeyCode::A).unwrap_or(false) {
                        apple.strafe(Vec2::new(-SPEED * delta_t, 0.0));
                    }
                    if  keyb.is_pressed(glutin::event::VirtualKeyCode::S).unwrap_or(false) {
                        apple.strafe(Vec2::new(0.00, -SPEED * delta_t));
                    }
                    if  keyb.is_pressed(glutin::event::VirtualKeyCode::D).unwrap_or(false) {
                        apple.strafe(Vec2::new(SPEED * delta_t, 0.0));
                    }
                    if keyb.is_pressed(glutin::event::VirtualKeyCode::Escape).unwrap_or(false){
                        *control_flow = ControlFlow::Exit;
                    }
                    // SPEED/ apple.get_size() will make sure the camera never falls more than SPEED distance from the apple apart
                    cam.lerp_to(apple.get_pos(), Vec2::new((SPEED/apple.get_size().x())*delta_t/1.5, (SPEED/apple.get_size().y())*delta_t), (SPEED/4.0)*delta_t/1.5/* snap range can't be bigger than speed otherwise the camera would just always snap to the target and since we know the target can only move 200.0*deltaT pixels we have to use this value as a max instead of any range we want in pixels*/);

                    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }

                    tile.bind_texture_for_sampling(test.get_program().get_sampler_id("obj_tex").unwrap());

                    test.bind().unwrap();
                    {
                        let i: i32 = i32::try_from(test.get_program().get_uniform_id("mvp").unwrap()).unwrap();
                        let m: Mat3 = proj * cam.get_mat().clone() * test.get_mat().clone();
                        //NOTE: to_cols_array consumes m so that's why we have t clone although hin this case it's kind of bas it's not the end of the world ( we probably wanted to change stuff ( multiply it by other matrices and change it's value before passing it ) and have our own mat anyway plus it's only like 9 floats )
                        test.get_program().set_uniform_mat3_f32(i, &m.to_cols_array());
                    }

                    test.render().unwrap();

                    t.bind_texture_for_sampling(apple.get_program().get_sampler_id("obj_tex").unwrap());

                    apple.bind().unwrap();
                    {
                        let i: i32 = i32::try_from(apple.get_program().get_uniform_id("mvp").unwrap()).unwrap();
                        let m: Mat3 = proj * cam.get_mat().clone() * apple.get_mat().clone();
                        //NOTE: to_cols_array consumes m so that's why we have t clone although hin this case it's kind of bas it's not the end of the world ( we probably wanted to change stuff ( multiply it by other matrices and change it's value before passing it ) and have our own mat anyway plus it's only like 9 floats )
                        apple.get_program().set_uniform_mat3_f32(i, &m.to_cols_array());
                    }

                    apple.render().unwrap();

                    gl_window.swap_buffers().unwrap();
                    start = Instant::now();
               }
            }
            _ => {}
        }

    });
}

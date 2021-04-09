use std::convert::TryInto;
use std::ptr;

use gl::types::*;
use gl_wrapper::render::program::Program;
use gl_wrapper::util::aggregator_obj::VAO;
use gl_wrapper::util::buffer_obj::{BOFunc, IBO, VBO};
use gl_wrapper::HasGLEnum;

pub struct Mesh<'a, IT>
where
    IT: HasGLEnum,
{
    vao: VAO,
    //attrib: Vec<&'a VBO<'a, AT>>,
    // /// Unused for now but might use in the future plus i want each mesh to be specific to the type of data used in attributes
    //attrib: PhantomData<AT>,
    indices: &'a IBO<IT>,
}

impl<'a, IT> Mesh<'a, IT>
where
    IT: HasGLEnum,
{
    pub fn new(vert_ord: &'a IBO<IT>) -> Option<Self> {
        let r = Mesh {
            vao: VAO::new(),
            indices: vert_ord,
        };
        Some(r)
    }

    pub fn adapt_bound_mesh_to_program(self: &mut Self, p: &Program) -> Result<(), ()> {
        self.vao.adapt_bound_vao_to_program(p)
    }

    pub fn adapt_bound_mesh_to_attrib<AT>(
        self: &mut Self,
        att: &VBO<AT>,
        att_loc: GLuint,
    ) -> Result<(), ()>
    where
        AT: HasGLEnum,
    {
        att.bind_bo();
        match self
            .vao
            .attach_bound_vbo_to_bound_vao(att, att_loc, 0, false)
        {
            Ok(()) => (),
            Err(_) => return Err(()),
        }

        Ok(())
    }

    pub fn bind_mesh(self: &Self) {
        self.vao.bind_ao();
        self.indices.bind_bo();
    }

    pub fn render_bound_mesh_with_bound_shader(self: &Self) -> Result<(), ()>
    where
        IT: HasGLEnum,
    {
        let s: i32 = match self.indices.get_size().try_into() {
            Ok(v) => v,
            Err(_) => return Err(()),
        };
        unsafe {
            gl::DrawElements(gl::TRIANGLES, s, IT::get_gl_type(), ptr::null());
        }
        Ok(())
    }
}

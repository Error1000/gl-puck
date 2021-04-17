use std::convert::TryInto;
use std::ptr;

use gl::types::*;
use gl_wrapper::render::program::{BoundProgram, Program};
use gl_wrapper::util::aggregator_obj::VAO;
use gl_wrapper::util::buffer_obj::{BOFunc, IBO, VBO};
use gl_wrapper::HasGLEnum;
use gl_wrapper::util::aggregator_obj::*;
use gl_wrapper::util::buffer_obj::*;

pub struct UnboundMesh<'a, IT>
where IT: HasGLEnum{
    unbound_vao: UnboundVAO,
    unbound_indicies: &'a UnboundIBO<IT>
}

impl<'a, IT> UnboundMesh<'a, IT>
where IT: HasGLEnum{
    pub fn new(vert_ord: &'a UnboundIBO<IT>) -> Self{
        Self{
            unbound_vao: VAO::new(),
            unbound_indicies: vert_ord
        }
    }

    pub fn bind<'b>(&'b mut self, bn1: &'b mut VAOBouncer, bn2: &'b mut IBOBouncer) -> BoundMesh<'b, IT>{
        // bind vao and ibo
        BoundMesh{
            vao: self.unbound_vao.bind_mut(bn1),
            indices: self.unbound_indicies.bind(bn2)
        }
    }

}

pub struct BoundMesh<'b, IT>
where
    IT: HasGLEnum,
{
    vao: MutBoundVAO<'b>,
    //attrib: Vec<&'a VBO<'a, AT>>,
    // /// Unused for now but might use in the future plus i want each mesh to be specific to the type of data used in attributes
    //attrib: PhantomData<AT>,
    indices: BoundIBO<'b, IT>,
}

impl<'a, 'b, IT> BoundMesh<'b, IT>
where
    IT: HasGLEnum,
{
    // pub fn new(vert_ord: &'a UnboundIBO<IT>) -> UnboundMesh<'a, IT> {
    //     UnboundMesh::from(Self {
    //         vao: VAO::new(),
    //         indices: vert_ord,
    //     })    
    // }

    
    pub fn adapt_mesh_to_program(self: &mut Self, p: &Program) -> Result<(), ()> {
        self.vao.adapt_vao_to_program(p)
    }

    pub fn adapt_mesh_to_attrib<AT>(
        self: &mut Self,
        att: &VBO<AT>,
        att_loc: GLuint,
    ) -> Result<(), ()>
    where
        AT: HasGLEnum,
    {
        match self
            .vao
            .attach_vbo_to_vao(att, att_loc, 0, false)
        {
            Ok(()) => (),
            Err(_) => return Err(()),
        }

        Ok(())
    }


    pub fn render_mesh_with_program(self: &Self, _prg: &Program) -> Result<(), ()>
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

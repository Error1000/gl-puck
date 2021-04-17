use std::ops::Bound;

use gl::types::*;
use gl_wrapper::render::program::{BoundProgram, Program};
use gl_wrapper::util::aggregator_obj::*;
use gl_wrapper::util::buffer_obj::*;
use gl_wrapper::HasGLEnum;
use glam::*;

pub trait World2D {
    fn get_mat(self: &mut Self) -> &Mat3;

    fn get_aabb(self: &Self) -> &Vec2;
    fn get_pos(self: &Self) -> &Vec2;
    fn set_pos(self: &mut Self, val: Vec2);
    fn strafe(self: &mut Self, val: Vec2);
}

pub trait World3D {
    fn get_mat(self: &mut Self) -> &Mat4;

    fn get_aabb(self: &Self) -> &Vec3;
    fn get_pos(self: &Self) -> &Vec3;
    fn set_pos(self: &mut Self, val: Vec3);
    fn strafe(self: &mut Self, val: Vec3);
}

pub trait Model {
    fn adapt_model_to_attrib<AT>(
        self: &mut Self,
        attrib: &VBO<AT>,
        attrib_loc: GLuint,
    ) -> Result<(), ()>
    where
        AT: HasGLEnum;
    fn adapt_model_to_program(self: &mut Self, p: &Program) -> Result<(), ()>;
    fn render(self: &Self, _prg: &Program) -> Result<(), ()>;
}

pub struct UnboundModel2D<'a, IT>
where 
    IT: HasGLEnum,
{
    pos: Vec2,
    aabb: Vec2,
    mat: Option<Mat3>,
    mesh: crate::mesh::UnboundMesh<'a, IT>
}

impl<'a, IT> UnboundModel2D<'a, IT>
where IT: HasGLEnum{
    pub fn new(m: crate::mesh::UnboundMesh<'a, IT>) -> Self{
        UnboundModel2D{
            pos: Vec2::ZERO,
            aabb: Vec2::ONE,
            mat: None,
            mesh: m
        }
    }

    pub fn bind<'b>(&'b mut self, bn1: &'b mut VAOBouncer, bn2: &'b mut IBOBouncer) -> BoundModel2D<'b, IT>{
        BoundModel2D{
            pos: &mut self.pos,
            aabb: &mut self.aabb,
            mat: &mut self.mat,
            mesh: self.mesh.bind(bn1, bn2)
        }
    }
    #[inline]
    fn update_mat(self: &mut Self) {
        self.mat = Some(Mat3::from_scale_angle_translation(self.aabb, 0.0, self.pos));
    }

    #[inline]
    pub fn set_size(self: &mut Self, val: Vec2) {
        self.aabb = val;
        self.mat = None;
    }

    #[inline]
    pub fn scale(self: &mut Self, val: Vec2) {
        self.aabb *= val;
        self.mat = None;
    }

    #[inline]
    pub fn get_size(self: &Self) -> &Vec2 {
        self.get_aabb()
    }

}

impl<IT> World2D for UnboundModel2D<'_, IT>
where IT: HasGLEnum{
    #[inline]
    fn get_mat(self: &mut Self) -> &Mat3 {
      if self.mat.is_none(){
          self.update_mat();
      }
      self.mat.as_ref().unwrap()
    }

    #[inline]
    fn get_aabb(self: &Self) -> &Vec2 {
        &self.aabb
    }

    #[inline]
    fn get_pos(self: &Self) -> &Vec2 {
        &self.pos
    }

    #[inline]
    fn set_pos(self: &mut Self, val: Vec2) {
        self.pos = val;
        self.mat = None;
    }

    #[inline]
    fn strafe(self: &mut Self, val: Vec2) {
        self.pos += val;
        self.mat = None;
    }
}

pub struct BoundModel2D<'b, IT>
where
    IT: HasGLEnum,
{
    pos:&'b mut  Vec2,
    aabb: &'b mut Vec2,
    mat: &'b mut Option<Mat3>,
    mesh: crate::mesh::BoundMesh<'b, IT>,
}

impl<'a, 'b, IT> BoundModel2D<'a, IT>
where
    IT: HasGLEnum,
{
    // pub fn new(m: crate::mesh::UnboundMesh<'a, IT>) -> Self {
    //     BoundModel2D {
    //         pos: Vec2::ZERO,
    //         aabb: Vec2::ONE,
    //         mat: None,
    //         mesh: m,
    //     }
    // }

    #[inline]
    fn update_mat(self: &mut Self) {
        *self.mat = Some(Mat3::from_scale_angle_translation(*self.aabb, 0.0, *self.pos));
    }

    #[inline]
    pub fn set_size(self: &mut Self, val: Vec2) {
        *self.aabb = val;
        *self.mat = None;
    }

    #[inline]
    pub fn scale(self: &mut Self, val: Vec2) {
        *self.aabb *= val;
        *self.mat = None;
    }

    #[inline]
    pub fn get_size(self: &Self) -> &Vec2 {
        self.get_aabb()
    }
}

impl<'a, 'b, IT> World2D for BoundModel2D<'a, IT>
where
    IT: HasGLEnum,
{
    #[inline]
    fn get_mat(self: &mut Self) -> &Mat3 {
        if self.mat.is_none() {
            self.update_mat();
        }
        self.mat.as_ref().unwrap()
    }

    #[inline]
    fn get_aabb(self: &Self) -> &Vec2 {
        &self.aabb
    }
    #[inline]
    fn get_pos(self: &Self) -> &Vec2 {
        &self.pos
    }
    #[inline]
    fn set_pos(self: &mut Self, val: Vec2) {
        *self.pos = val;
        *self.mat = None;
    }
    #[inline]
    fn strafe(self: &mut Self, val: Vec2) {
        *self.pos += val;
        *self.mat = None;
    }
}

impl<'a, 'b, IT> Model for BoundModel2D<'a, IT>
where
    IT: HasGLEnum,
{
    fn adapt_model_to_attrib<AT>(
        self: &mut Self,
        attrib: &VBO<AT>,
        attrib_loc: GLuint,
    ) -> Result<(), ()>
    where
        AT: HasGLEnum,
    {
        self.mesh
            .adapt_mesh_to_attrib::<AT>(attrib, attrib_loc)
    }

    #[inline(always)]
    fn adapt_model_to_program(self: &mut Self, p: &Program) -> Result<(), ()> {
        self.mesh.adapt_mesh_to_program(p)
    }

    // #[inline(always)]
    // fn bind_model(self: &Self) {
    //     self.mesh.bind_mesh();
    // }

    #[inline(always)]
    fn render(self: &Self, _prg: &Program) -> Result<(), ()> {
        self.mesh.render_mesh_with_program(_prg)?;
        Ok(())
    }
}



/* 
pub struct Model3D<'a, IT>
where
    IT: HasGLEnum,
{
    pos: Vec3,
    aabb: Vec3,
    mat: Option<Mat4>,
    mesh: crate::mesh::Mesh<'a, IT>,
}

impl<'a, 'b, IT> Model3D<'a, IT>
where
    IT: HasGLEnum,
{
    pub fn new(m: crate::mesh::Mesh<'a, IT>) -> Self {
        Model3D {
            pos: Vec3::ZERO,
            aabb: Vec3::ONE,
            mat: None,
            mesh: m,
        }
    }

    #[inline]
    fn update_mat(self: &mut Self) {
        self.mat = Some(Mat4::from_scale_rotation_translation(
            self.aabb,
            Quat::IDENTITY,
            self.pos,
        ));
    }

    #[inline]
    pub fn set_size(self: &mut Self, val: Vec3) {
        self.aabb = val;
        self.mat = None;
    }

    #[inline]
    pub fn scale(self: &mut Self, val: Vec3) {
        self.aabb *= val;
        self.mat = None;
    }

    #[inline]
    pub fn get_size(self: &Self) -> &Vec3 {
        self.get_aabb()
    }
}

impl<'a, 'b, IT> World3D for Model3D<'a, IT>
where
    IT: HasGLEnum,
{
    #[inline]
    fn get_mat(self: &mut Self) -> &Mat4 {
        if self.mat.is_none() {
            self.update_mat();
        }
        self.mat.as_ref().unwrap()
    }

    #[inline]
    fn get_aabb(self: &Self) -> &Vec3 {
        &self.aabb
    }
    #[inline]
    fn get_pos(self: &Self) -> &Vec3 {
        &self.pos
    }
    #[inline]
    fn set_pos(self: &mut Self, val: Vec3) {
        self.pos = val;
        self.mat = None;
    }
    #[inline]
    fn strafe(self: &mut Self, val: Vec3) {
        self.pos += val;
        self.mat = None;
    }
}

impl<'a, 'b, IT> Model for Model3D<'a, IT>
where
    IT: HasGLEnum,
{
    // Make sure there is no overhead in passing variables
    #[inline(always)]
    fn adapt_bound_model_to_attrib<AT>(
        self: &mut Self,
        attrib: &VBO<AT>,
        attrib_loc: GLuint,
    ) -> Result<(), ()>
    where
        AT: HasGLEnum,
    {
        self.mesh.adapt_bound_mesh_to_attrib(attrib, attrib_loc)
    }

    fn adapt_bound_model_to_program(self: &mut Self, p: &Program) -> Result<(), ()> {
        self.mesh.adapt_bound_mesh_to_program(p)
    }

    fn bind_model(self: &Self) {
        self.mesh.bind_mesh();
    }

    #[inline]
    fn render(self: &Self, _prg: &BoundProgram) -> Result<(), ()> {
        self.mesh.render_bound_mesh_with_program(_prg)?;
        Ok(())
    }
}
*/
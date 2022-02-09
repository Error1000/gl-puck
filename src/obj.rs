use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    str::FromStr, borrow::Borrow,
    hash::Hash, convert::TryFrom
};

use objld::LineResult;
use rayon::prelude::*;


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Dimension{OneDim = 1, TwoDim = 2, ThreeDim = 3}

pub struct VertexAttribs<T> {
    data: Vec<T>,
    elem_per_vert: Dimension
}

impl<T> VertexAttribs<T> 
where T: Copy + Default {
    pub fn new(elem_per_vert: Dimension) -> Self{
        Self{
            data: Vec::new(),
            elem_per_vert
        }
    }

    pub fn get_vals(&mut self) -> &mut Vec<T>{
        &mut self.data
    }

    pub fn push1d(&mut self, element: T){
        assert!(self.elem_per_vert == Dimension::OneDim);
        self.data.push(element);
    }

    pub fn push2d(&mut self, element: (T, T)){
        assert!(self.elem_per_vert == Dimension::TwoDim);
        self.data.push(element.0);
        self.data.push(element.1);
    }

    pub fn push3d(&mut self, element: (T, T, T)){
        assert!(self.elem_per_vert == Dimension::ThreeDim);
        self.data.push(element.0);
        self.data.push(element.1);
        self.data.push(element.2);  
    }

    pub fn get_elem_per_vert(&self) -> Dimension { self.elem_per_vert }

    pub fn get(&self, ind: usize) -> Vec<T>{
        match self.elem_per_vert{
            Dimension::OneDim => vec![self.data[ind]],
            Dimension::TwoDim => vec![self.data[ind*2], self.data[ind*2+1]],
            Dimension::ThreeDim => vec![self.data[ind*3], self.data[ind*3+1], self.data[ind*3+2]],
        }
    }

    pub fn set(&mut self, ind: usize, val: Vec<T>){
        match self.elem_per_vert{
            Dimension::OneDim => { self.data[ind] = val[0]; },
            Dimension::TwoDim => { self.data[ind*2] = val[0]; self.data[ind*2+1] = val[1]; },
            Dimension::ThreeDim => {self.data[ind*3] = val[0]; self.data[ind*3+1] = val[1]; self.data[ind*3+2] = val[2];}
        }
    }

    pub fn resize_to(&mut self, len: usize){
        match self.elem_per_vert{
            Dimension::OneDim => self.data.resize(len, T::default()),
            Dimension::TwoDim => self.data.resize(len*2, T::default()),
            Dimension::ThreeDim => self.data.resize(len*3, T::default())
        }
    }

    pub fn len(&self) -> usize{
        match self.elem_per_vert{
            Dimension::OneDim => self.data.len(),
            Dimension::TwoDim => self.data.len()/2,
            Dimension::ThreeDim => self.data.len()/3,
        }
    }
}

pub struct ObjData<T, I> {
    pub pos_data: VertexAttribs<T>,
    pub tex_data: Option<VertexAttribs<T>>,
    pub norm_data: Option<VertexAttribs<T>>,
    pub indicies: Vec<I>,
}

#[derive(Eq, PartialEq, Hash)]
struct Vertex<T>{
    pub pos: Vec<T>,
    pub norm: Option<Vec<T>>,
    pub tex: Option<Vec<T>>
}

impl<T, I> ObjData<T, I>
where T: FromStr + Send + Copy+ PartialEq + Hash + Eq + Default, I: FromStr + Copy + Send + TryFrom<usize>{
    pub fn new(pos_data_dim: Dimension, tex_data_dim: Option<Dimension>, norm_data_dim: Option<Dimension>) -> Self{
        Self{
            pos_data: VertexAttribs::new(pos_data_dim),
            tex_data: if let Some(d) = tex_data_dim { Some(VertexAttribs::new(d)) } else {None},
            norm_data: if let Some(d) = norm_data_dim { Some(VertexAttribs::new(d))} else {None},
            indicies: Vec::new()
        }
    }

    pub fn load(&mut self, f: &mut File) -> io::Result<()>{
        let parsed: Vec<LineResult<T, isize>> = {
            let mut lines = String::new();
            f.read_to_string(&mut lines)?;
            let r = objld::parse_file(lines.borrow()).collect();
            drop(lines);
            r
        };

        let mut vert_ind: Vec<objld::VertexIndeces<isize>> = Vec::new();
        for line in parsed{
            match line{
                    LineResult::FaceLine(f) => {
                            match f{
                                objld::Face::Face3 { v1, v2, v3 } => { 
                                    vert_ind.push(v1); vert_ind.push(v2); vert_ind.push(v3); 
                                },
                                objld::Face::Face4 { v1, v2, v3, v4 } => {
                                    vert_ind.push(v1); vert_ind.push(v2); vert_ind.push(v3); // First triangle of square 
                                    vert_ind.push(v3); vert_ind.push(v4); vert_ind.push(v1); // Second triangle of square
                                }
                            }
                    },
                    LineResult::VertDataLine(v) => {
                            match v{
                                objld::VertexData::Coord2 { x, y } if self.pos_data.get_elem_per_vert() == Dimension::TwoDim => self.pos_data.push2d((x, y)),
                                objld::VertexData::Coord3 { x, y, z } if self.pos_data.get_elem_per_vert() == Dimension::ThreeDim => self.pos_data.push3d((x, y, z)),
                                objld::VertexData::Normal { x, y, z } => {
                                    if let Some(n) = &mut self.norm_data{
                                        if n.get_elem_per_vert() == Dimension::ThreeDim {
                                            n.push3d((x, y, z));
                                        }
                                    }
                                },
                                objld::VertexData::TextureCoord3 { u, v, w } => {
                                    if let Some(t) = &mut self.tex_data{
                                        if t.get_elem_per_vert() == Dimension::ThreeDim {
                                            t.push3d((u, v, w));
                                        }
                                    }
                                },
                                objld::VertexData::TextureCoord2 { u, v } => {
                                    if let Some(t) = &mut self.tex_data{
                                        if t.get_elem_per_vert() == Dimension::TwoDim {
                                            t.push2d((u, v));
                                        }
                                    } 
                                },
                                objld::VertexData::TextureCoord1 { u } => {
                                    if let Some(t) = &mut self.tex_data{
                                        if t.get_elem_per_vert() == Dimension::OneDim {
                                            t.push1d(u);
                                        }
                                    } 
                                },
                                _ => {}
                            }
                    },
                    LineResult::Error(_e) =>{ }, // FIXME: Should find a way to not avoid errors
                    LineResult::NoData => {}
            }
        }

        let mut h: HashMap<Vertex<T>, usize> = HashMap::new();
        let mut curr_ind: usize = 0;

        for v in vert_ind {
            let process_index = |ind: isize, data_len: usize| -> usize{
                if ind < 0 { (data_len as isize + ind) as usize } else {ind as usize}
            };
    
            let loaded_vert = Vertex::<T>{
                pos: self.pos_data.get(process_index(v.coord_rindex, self.pos_data.len())),
                norm: if let Some(norm_index) = v.normal_rindex{
                    if let Some(norms) = &mut self.norm_data{
                        Some(norms.get(process_index(norm_index, norms.len())))
                    }else {None}
                } else {None},
                tex: if let Some(tex_index) = v.texcoord_rindex{
                    if let Some(texs) = &mut self.tex_data{
                        Some(texs.get(process_index(tex_index, texs.len())))
                    }else {None}
                } else {None},
            };
            if let Some(already_existing_ind) = h.get(&loaded_vert){
                self.indicies.push(I::try_from(*already_existing_ind).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Indeces too big!"))?);
            }else{
                h.insert(loaded_vert, curr_ind);
                self.indicies.push(I::try_from(curr_ind).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Indeces too big!"))?);
                curr_ind += 1;
            }
        }
        // Clean up
        self.pos_data = VertexAttribs::new(self.pos_data.get_elem_per_vert());
        self.norm_data = self.norm_data.as_ref().map(|inner|VertexAttribs::<T>::new(inner.get_elem_per_vert()));
        self.tex_data = self.tex_data.as_ref().map(|inner|VertexAttribs::<T>::new(inner.get_elem_per_vert()));

        self.pos_data.resize_to(h.len());
        self.norm_data.as_mut().map(|v|{v.resize_to(h.len()); v});
        self.tex_data.as_mut().map(|v|{v.resize_to(h.len()); v});

        for (v, i) in h {
            self.pos_data.set(i, v.pos);
            if let Some(texs) = &mut self.tex_data{
                if let Some(new_tex) = v.tex{
                    texs.set(i, new_tex);
                }              
            }

            if let Some(norms) = &mut self.norm_data{
                if let Some(new_norm) = v.norm{
                    norms.set(i, new_norm);
                }              
            }
        }
        Ok(())
    }
}
 
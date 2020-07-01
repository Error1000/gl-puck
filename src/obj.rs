use gl_wrapper::unwrap_option_or_ret;
use gl_wrapper::unwrap_result_or_ret;
use itertools::izip;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Error, ErrorKind};
use std::str::FromStr;

pub struct VertexAttrib<T> {
    elem_per_vert: u8,
    attribs: Vec<T>,
    token: &'static str,
}

impl<T> VertexAttrib<T> {
    pub fn new(elem_per_vert: u8, token: &'static str) -> Self {
        if elem_per_vert == 0 {
            panic!("Number of elements per vertex cannot be 0!");
        }
        VertexAttrib {
            elem_per_vert,
            attribs: Vec::<T>::new(),
            token,
        }
    }

    pub fn len(self: &Self) -> usize {
        self.attribs.len()
    }

    pub fn is_empty(self: &Self) -> bool {
        self.len() == 0
    }
    pub fn get_vals(self: &mut Self) -> &mut Vec<T> {
        &mut self.attribs
    }

    pub fn get_token(self: &Self) -> &'static str {
        &self.token
    }
    pub fn get_elem_per_vert(self: &Self) -> u8 {
        self.elem_per_vert
    }
}

pub struct ObjData<T, U> {
    pub attribs: Vec<VertexAttrib<T>>,
    pub indicies: Vec<U>,
}

impl<T, U> ObjData<T, U>
where
    T: FromStr + Copy + Default + Debug,
    U: FromStr + Copy + Default + TryFrom<usize>,
{
    pub fn new(attribs: Vec<VertexAttrib<T>>) -> Self {
        ObjData::<T, U> {
            attribs,
            indicies: Vec::new(),
        }
    }

    fn compute_face(
        self: &Self,
        indices: Vec<&str>,
        ordered_data: &mut ObjData<T, U>,
        new_ind: &mut usize,
    ) -> io::Result<()> {
        for (index_to_parse, current_attrib, new_attrib) in
            izip!(indices, &self.attribs, &mut ordered_data.attribs)
        {
            if index_to_parse.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Empty ( missing ) index with token: {}!",
                        current_attrib.token
                    ),
                ));
            }

            let e = Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Parsing failed for index with token: {}!",
                    current_attrib.token
                ),
            ));
            let iind = if index_to_parse.starts_with('-') {
                (current_attrib.len() / current_attrib.elem_per_vert as usize)
                    - unwrap_result_or_ret!(index_to_parse[1..].parse::<usize>(), e)
            } else {
                unwrap_result_or_ret!(index_to_parse.parse::<usize>(), e) - 1
            };

            if new_attrib.len() <= (*new_ind + 1) * current_attrib.elem_per_vert as usize {
                new_attrib.attribs.resize(
                    (*new_ind + 1) * current_attrib.elem_per_vert as usize,
                    Default::default(),
                );
            }
            for jij in 0..current_attrib.elem_per_vert as usize {
                new_attrib.attribs[*new_ind * current_attrib.elem_per_vert as usize + jij] = *unwrap_option_or_ret!(
                    current_attrib
                        .attribs
                        .get(iind * current_attrib.elem_per_vert as usize + jij),
                    Err(Error::new(ErrorKind::InvalidData, "Index out of bounds!"))
                );
            }
        }

        ordered_data.indicies.push(match U::try_from(*new_ind) {
            Ok(val) => val,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Data from file does not fit in index type, index:{}!",
                        *new_ind
                    ),
                ))
            }
        });

        *new_ind += 1;
        Ok(())
    }
    /// NOTE: For some reason turning optimisations on makes this go much, much faster so as a tem pworkaround until either the language gets better optimisations or i improve my code, the debug builds are set to optimisation level 2
    pub fn load(self: &mut Self, f: &File) -> io::Result<Vec<String>> {
        let reader = BufReader::new(f);
        let mut v: Vec<VertexAttrib<T>> = Vec::new();
        for a in &self.attribs {
            v.push(VertexAttrib::new(a.elem_per_vert, a.token));
        }
        let mut ordered_data = ObjData::<T, U>::new(v);
        let mut unrecognised_tokens = Vec::<String>::new();

        let mut global_ind: usize = 0;
        for line in reader.lines() {
            let unwrapped_line = String::from(line?.trim());
            if unwrapped_line.is_empty() {
                continue;
            }
            let mut ar = unwrapped_line.split_whitespace();

            let first_word = unwrap_option_or_ret!(
                ar.next(),
                Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Expected identifier got eof!"
                ))
            )
            .trim();

            if first_word == "#" {
                continue;
            }
            let mut recon = false;

            for attrib in &mut self.attribs {
                if first_word == attrib.token {
                    recon = true;
                    for _ in 0..attrib.elem_per_vert {
                        let nxt = unwrap_option_or_ret!(
                            ar.next(),
                            Err(Error::new(
                                ErrorKind::UnexpectedEof,
                                "Expected vertex data got eof!"
                            ))
                        );
                        attrib.attribs.push(match nxt.parse::<T>() {
                            Ok(val) => val,
                            Err(_) => {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    format!(
                                        "Invalid vertex position value, token:{}!",
                                        attrib.token
                                    ),
                                ))
                            }
                        });
                    }
                }
            }

            if first_word == "f" {
                recon = true;
                for a in &self.attribs {
                    if a.is_empty() {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "No indices in file with token: {} even though requested!",
                                a.token
                            ),
                        ));
                    }
                }

                let data = ar.collect::<Vec<&str>>();
                if data.len() != 3 && data.len() != 4 {
                    return Err(Error::new(ErrorKind::InvalidData, "Malformed face, this loader only supports 3 vertices or 4 vertices per face!"));
                }

                for vert in [data[0], data[1], data[2]].iter_mut() {
                    self.compute_face(
                        vert.split('/').collect::<Vec<&str>>(),
                        &mut ordered_data,
                        &mut global_ind,
                    )?;
                    global_ind += 1;
                }
                if data.len() == 4 {
                    for vert in [data[0], data[3], data[2]].iter_mut() {
                        self.compute_face(
                            vert.split('/').collect::<Vec<&str>>(),
                            &mut ordered_data,
                            &mut global_ind,
                        )?;
                        global_ind += 1;
                    }
                }
            }
            if !recon {
                unrecognised_tokens.push(String::from(first_word));
                //eprintln!("Unrecognised token in obj file: {}, skipping line: \"{}\" ...", first_word, unwrapped_line);
            }
        }
        *self = ordered_data;
        Ok(unrecognised_tokens)
    }

    pub fn dedup(self: &mut Self) {
        unimplemented!();
    }
}

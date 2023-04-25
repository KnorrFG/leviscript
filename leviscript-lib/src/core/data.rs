use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Data {
    String(String),
    Int(i64),
    Vec(Vec<Data>),
    Ref(DataRef),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataRef {
    StackIdx(usize),
    DataSectionIdx(usize),
}

pub trait DataAs<'a>: std::fmt::Debug + Sized {
    fn get_as(data: &'a Data) -> Option<Self>;
}

impl Data {
    pub fn get_as<'a, T: DataAs<'a>>(&'a self) -> Option<T> {
        <T as DataAs>::get_as(&self)
    }

    pub fn offset_data_section_addr(&mut self, offset: usize) {
        use Data::*;
        match self {
            String(_) | Int(_) => {}
            Vec(ds) => {
                for d in ds {
                    d.offset_data_section_addr(offset);
                }
            }
            Ref(r) => {
                if let DataRef::DataSectionIdx(i) = r {
                    *i += offset;
                }
            }
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Data::String(s) => Some(s.clone()),
            Data::Int(i) => Some(i.to_string()),
            Data::Vec(_) | Data::Ref(_) => None,
        }
    }
}

impl<'a> DataAs<'a> for &'a str {
    fn get_as(data: &'a Data) -> Option<&'a str> {
        if let Data::String(s) = data {
            Some(&s)
        } else {
            None
        }
    }
}

impl DataRef {
    pub fn offset_data_section_addr(&mut self, offset: usize) {
        if let Self::DataSectionIdx(i) = self {
            *i += offset;
        }
    }
}

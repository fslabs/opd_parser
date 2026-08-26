mod parser;

pub use parser::parse;

use serde::{Deserialize, Serialize};

pub struct OpdFile {
    pub header: OpdHeader,
    pub centroids: Vec<Centroid>,
    pub frames: Frames,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct OpdHeader {
    pub version: String,
    pub compressed: Option<String>,
    #[serde(rename = "type")]
    pub ty: String,
    pub directive: OpdHeaderDirective,
}

#[derive(Clone)]
pub struct Frame<T> {
    pub time: f32,
    pub data: Vec<T>,
}

impl<'a, T> IntoIterator for &'a Frame<T>
where
    T: Copy + Into<f32>,
{
    type IntoIter = FrameIterator<'a, T>;
    type Item = [f32; 3];

    fn into_iter(self) -> Self::IntoIter {
        FrameIterator {
            iter: self.data.iter(),
        }
    }
}

pub struct FrameIterator<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T: Copy> Iterator for FrameIterator<'a, T>
where
    T: Into<f32>,
{
    type Item = [f32; 3];

    fn next(&mut self) -> Option<Self::Item> {
        let max_value: usize = (1 << (std::mem::size_of::<T>() * 8 - 1)) - 1;
        let max_value = max_value as f32;
        let arr: [f32; 3] = [
            {
                let a = self.iter.next()?;
                (*a).into() / max_value
            },
            {
                let a = self.iter.next()?;
                (*a).into() / max_value
            },
            {
                let a = self.iter.next()?;
                (*a).into() / max_value
            },
        ];
        Some(arr)
    }
}

#[derive(Clone)]
pub enum Frames {
    I8(Vec<Frame<i8>>),
    I16(Vec<Frame<i16>>),
    I32(Vec<Frame<i32>>),
    I64(Vec<Frame<i64>>),
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct FrameMeta {
    pub time: f32,
    pub offset: usize,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct OpdHeaderDirective {
    pub version: String,
    pub meta: OpdHeaderDirectiveMeta,

    #[serde(rename = "numCentroids")]
    pub num_centroids: Option<usize>,
    #[serde(rename = "hasCentroidVolumes")]
    pub has_centroid_volumes: Option<bool>,

    #[serde(rename = "numPoints")]
    pub num_points: Option<usize>,

    pub origin: OpdHeaderDirectiveOrigin,

    pub precision: usize,
    pub scale: [f32; 3],
    pub frames: Vec<FrameMeta>,

    pub index: Option<bool>,
    #[serde(rename = "subCentroids")]
    pub sub_centroids: Option<bool>,

    #[serde(rename = "lastFrameCorrected")]
    pub last_frame_corrected: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct OpdHeaderDirectiveOrigin {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<OpdHeaderDirectiveOrigin> for [f64; 3] {
    fn from(value: OpdHeaderDirectiveOrigin) -> Self {
        [value.x, value.y, value.z]
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct OpdHeaderDirectiveMeta {
    #[serde(rename = "projectId")]
    pub project_id: String,

    #[serde(rename = "projectName")]
    pub project_name: String,
}

pub struct Centroid {
    pub parent_id: u32,

    /// Relative to origin defined in header
    pub offset: [f32; 3],
}

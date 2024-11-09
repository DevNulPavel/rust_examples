use serde::{Deserialize, Serialize};
use nalgebra::DMatrix;

#[derive(Serialize, Deserialize)]
pub struct Data {
    p: u32,
    open_key: MatrixWrapper,
    close_key: MatrixWrapper,
}

impl Data {
    pub fn new(p: u32, open_key: MatrixWrapper, close_key: MatrixWrapper) -> Self {
        Data {
            p,
            open_key,
            close_key
        }
    }
    pub fn get_open_key(&self) -> MatrixWrapper {
        self.open_key.clone()
    }
    pub fn get_close_key(&self) -> MatrixWrapper {
        self.close_key.clone()
    }
    pub fn get_p(&self) -> u32 {
        self.p
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MatrixWrapper {
    rows: usize,
    cols: usize,
    data: Vec<u32>,
}

impl From<&DMatrix<u32>> for MatrixWrapper {
    fn from(value: &DMatrix<u32>) -> Self {
        MatrixWrapper {
            rows: value.nrows(),
            cols: value.ncols(),
            data: value.iter().cloned().collect(),
        }
    }
}

impl Into<DMatrix<u32>> for MatrixWrapper {
    fn into(self) -> DMatrix<u32> {
        DMatrix::from_vec(self.rows, self.cols, self.data)
    }
}

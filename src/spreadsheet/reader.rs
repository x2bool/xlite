use calamine::{DataType};
use std::vec::IntoIter;

pub struct DataReader {
    rows: IntoIter<Vec<DataType>>,
    row: Option<Vec<DataType>>,
    rowid: u32,
}

impl DataReader {
    pub fn new(mut rows: IntoIter<Vec<DataType>>, rowid: u32) -> Self {
        let row = rows.next();
        Self {
            rows,
            row,
            rowid,
        }
    }

    pub fn has_value(&self) -> bool {
        self.row.is_some()
    }

    pub fn get_value(&self, i: usize) -> Option<DataType> {
        if let Some(ref row) = self.row {
            if i < row.len() {
                let col = row[i].clone();
                return Some(col);
            }
        }
        None
    }

    pub fn get_rowid(&self) -> u32 {
        self.rowid
    }

    pub fn move_next(&mut self) {
        self.row = self.rows.next();

        if self.row.is_some() {
            self.rowid += 1;
        }
    }
}

use calamine::{DataType, Range, Rows};
use std::mem::transmute;

pub struct DataReader {
    range: Range<DataType>,
    state: DataReaderState<'static>,
}

struct DataReaderState<'a> {
    rows: Rows<'a, DataType>,
    row: Option<&'a [DataType]>,
    rowid: u32,
}

impl DataReader {
    pub fn new(range: Range<DataType>) -> Self {
        let mut rows = range.rows();
        let row = rows.next();
        let rowid = range.start().unwrap_or((0, 0)).0;

        // transmute to static because we have the self-referencing struct
        let rows = unsafe {
            transmute::<Rows<'_, DataType>, Rows<'static, DataType>>(rows)
        };
        let row = unsafe {
            transmute::<Option<&'_ [DataType]>, Option<&'static [DataType]>>(row)
        };

        DataReader {
            range,
            state: DataReaderState { rows, row, rowid },
        }
    }

    pub fn has_value(&self) -> bool {
        self.state.row.is_some()
    }

    pub fn get_value(&self, i: usize) -> Option<&DataType> {
        if let Some(ref row) = self.state.row {
            if i < row.len() {
                let col = &row[i];
                return Some(col);
            }
        }
        None
    }

    pub fn get_rowid(&self) -> u32 {
        self.state.rowid
    }

    pub fn move_next(&mut self) {
        self.state.row = self.state.rows.next();

        if self.state.row.is_some() {
            self.state.rowid += 1;
        }
    }
}

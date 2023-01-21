use crate::options::UsingOption;
use crate::spreadsheet::{
    cells::{CellIndex, CellRange},
    reader::DataReader,
};
use calamine::{open_workbook_auto, DataType, Range, Reader, Sheets};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

pub struct DataManager {
    sheets: Sheets<BufReader<File>>,
    worksheet: String,
    range: Option<CellRange>,
    colnames_row: Option<u32>,
}

pub enum DataManagerError {
    NoFilename,
    NoWorksheet,
    Calamine(calamine::Error),
}

impl DataManager {
    pub fn get_sheets(&mut self) -> &mut Sheets<BufReader<File>> {
        &mut self.sheets
    }

    pub fn get_effective_range(&mut self) -> Range<DataType> {
        let range = self.sheets.worksheet_range(self.worksheet.as_str());
        if let Some(Ok(r)) = range {
            match self.range {
                Some(sub) => {
                    let start = sub.get_start();
                    let mut end = sub.get_end();

                    if end.get_y() == 0 {
                        end = CellIndex::new(end.get_x(), r.height() as u32)
                    }

                    r.range(start.to_zero_indexed(), end.to_zero_indexed())
                }
                None => r,
            }
        } else {
            Range::empty()
        }
    }

    pub fn get_columns(&mut self) -> Vec<String> {
        let range = self.get_effective_range();
        if range.get_size().1 > 0 {
            let row_workspace_sheet = self.colnames_row
                .and_then(|v| Some((v, self.sheets.worksheet_range(self.worksheet.as_str()))))
                .and_then(|(row, sheet)| Some((row, sheet?.ok()?)));
            (range.start().unwrap().1..=range.end().unwrap().1)
                .into_iter()
                .map(|n| {
                    row_workspace_sheet
                        .as_ref()
                        .and_then(|(row, sheet)| sheet.get_value((*row, n)).map(|v| v.to_string()))
                        .unwrap_or_else(|| CellIndex::new(n + 1, 1).get_x_as_string())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn read(&mut self) -> DataReader {
        let range = self.get_effective_range();

        DataReader::new(range)
    }
}

#[derive(Default)]
pub struct DataManagerBuilder {
    file: Option<String>,
    worksheet: Option<String>,
    range: Option<CellRange>,
    colnames_row: Option<u32>,
}

impl DataManagerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_options(options: Vec<UsingOption>) -> Self {
        let mut builder = Self::new();

        for option in options {
            match option {
                UsingOption::File(file) => {
                    builder = builder.file(file);
                }
                UsingOption::Worksheet(worksheet) => {
                    builder = builder.worksheet(worksheet);
                }
                UsingOption::Range(range) => {
                    builder = builder.range(CellRange::try_parse(range.as_str()).unwrap());
                }
                UsingOption::ColNames(colnames) => {
                    // We substract 1 to go from excel indexing (which starts at 1) to 0-based
                    // indexing of the row.
                    builder = builder.colnames_row(u32::from_str(colnames.as_str()).unwrap().saturating_sub(1));
                },
            }
        }

        builder
    }

    pub fn file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn worksheet(mut self, name: String) -> Self {
        self.worksheet = Some(name);
        self
    }

    pub fn range(mut self, range: CellRange) -> Self {
        self.range = Some(range);
        self
    }

    pub fn colnames_row(mut self, row: u32) -> Self {
        self.colnames_row = Some(row);
        self
    }

    pub fn open(self) -> Result<DataManager, DataManagerError> {
        if let Some(file) = self.file {
            if let Some(worksheet) = self.worksheet {
                match open_workbook_auto(Path::new(&file)) {
                    Ok(sheets) => Ok(DataManager {
                        sheets,
                        worksheet,
                        range: self.range,
                        colnames_row: self.colnames_row,
                    }),
                    Err(err) => Err(DataManagerError::Calamine(err)),
                }
            } else {
                Err(DataManagerError::NoWorksheet)
            }
        } else {
            Err(DataManagerError::NoFilename)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CellIndex {
    x: u32,
    y: u32,
}

impl CellIndex {

    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn try_parse(s: &str) -> Option<Self> {
        let chars = s.chars();

        let mut alpha = String::new();
        let mut num = String::new();

        for ch in chars {
            if ch.is_ascii_alphabetic() {
                if num.len() == 0 {
                    alpha.push(ch);
                } else {
                    return None;
                }
            } else if ch.is_ascii_digit() {
                num.push(ch);
            }
        }

        if alpha.len() > 0 {
            let x = column_to_index(alpha.to_uppercase().as_str());
            if num.len() > 0 {
                let y = row_to_index(num.as_str());
                Some(CellIndex::new(x, y))
            } else {
                Some(CellIndex::new(x, 0))
            }
        } else {
            None
        }
    }

    pub fn get_x(&self) -> u32 {
        self.x
    }

    pub fn get_y(&self) -> u32 {
        self.y
    }

    pub fn get_x_as_string(&self) -> String {
        index_to_column(self.x)
    }

    pub fn get_y_as_string(&self) -> String {
        index_to_row(self.y)
    }

    pub fn to_zero_indexed(&self) -> (u32, u32) {
        (if self.y > 0 { self.y - 1 } else { 0 },
            if self.x > 0 { self.x - 1 } else { 0 })
    }

}

#[derive(Debug, Copy, Clone)]
pub struct CellRange {
    start: CellIndex,
    end: CellIndex,
}

impl CellRange {

    pub fn new(start: CellIndex, end: CellIndex) -> Self {
        Self { start, end }
    }

    pub fn try_parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(":").collect();
        if parts.len() == 2 {
            let start = CellIndex::try_parse(parts[0])?;
            let end = CellIndex::try_parse(parts[1])?;
            Some(CellRange::new(start, end))
        } else {
            None
        }
    }

    pub fn get_start(&self) -> CellIndex {
        self.start
    }

    pub fn get_end(&self) -> CellIndex {
        self.end
    }

}

fn column_to_index(column: &str) -> u32 {
    let column = column.as_bytes();
    let mut sum = 0;
    let len = column.len();
    for i in 0..len {
        let c = column[len - i - 1];
        sum += (c - b'A' + 1) as u32 * 26u32.pow(i as u32);
    }
    sum
}

fn index_to_column(index: u32) -> String {
    format!("{}", (b'A' + ((index - 1) as u8)) as char) // TODO: multiple chars
}

fn row_to_index(row: &str) -> u32 {
    row.parse::<u32>().unwrap()
}

fn index_to_row(index: u32) -> String {
    (index + 1).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_to_index_gives_correct_value_for_a() {
        let index = column_to_index("A");
        assert_eq!(index, 1);
    }

    #[test]
    fn column_to_index_gives_correct_value_for_z() {
        let index = column_to_index("Z");
        assert_eq!(index, 26);
    }

    #[test]
    fn index_to_column_gives_correct_value_for_0() {
        let column = index_to_column(1);
        assert_eq!(column, "A");
    }

    #[test]
    fn index_to_column_gives_correct_value_for_25() {
        let column = index_to_column(26);
        assert_eq!(column, "Z")
    }

    #[test]
    fn try_parse_cell_index_from_single_letter() {
        let index = CellIndex::try_parse("A").unwrap();
        assert_eq!(index.x, 1);
        assert_eq!(index.y, 0);
    }

    #[test]
    fn try_parse_cell_index_from_letter_and_number() {
        let index = CellIndex::try_parse("A1").unwrap();
        assert_eq!(index.x, 1);
        assert_eq!(index.y, 1);
    }

    #[test]
    fn try_parse_cell_range_from_letter_and_number() {
        let range = CellRange::try_parse("A1:Z9").unwrap();
        assert_eq!(range.start.x, 1);
        assert_eq!(range.start.y, 1);
        assert_eq!(range.end.x, 26);
        assert_eq!(range.end.y, 9);
    }

    #[test]
    fn to_zero_indexed_gives_0_indexed_tuple_in_y_x_format() {
        let index = CellIndex::new(1, 9);
        let tuple = index.to_zero_indexed();
        assert_eq!(tuple.0, 8);
        assert_eq!(tuple.1, 0);
    }
}

use rusqlite::{Connection, params};

#[cfg(target_os = "linux")]
static LIB_PATH: &str = "./target/debug/libxlite.so";

#[cfg(target_os = "macos")]
static LIB_PATH: &str = "./target/debug/libxlite.dylib";

#[cfg(target_os = "windows")]
static LIB_PATH: &str = "./target/debug/xlite.dll";

fn init_connection() -> Connection {
    let connection = Connection::open_in_memory().unwrap();

    unsafe {
        connection.load_extension_enable().unwrap();
        connection.load_extension(LIB_PATH, None).unwrap();
        connection.load_extension_disable().unwrap();
    }

    connection
}

#[derive(PartialEq, Debug)]
struct Abcd {
    alpha: String,
    number: f32,
    word: String,
    kind: String,
}

#[derive(PartialEq, Debug)]
struct AbcdAgg {
    kind: String,
    count: i32,
}

#[derive(PartialEq, Debug)]
struct Bc {
    number: f32,
    word: String,
}

#[test]
fn test_load_extension() {
    init_connection();
}

#[test]
fn test_abcdef_file_simple() {
    let connection = init_connection();
    connection.execute("\
        CREATE VIRTUAL TABLE test_data USING xlite(\
            FILENAME './tests/abcdef.xlsx',\
            WORKSHEET 'Sheet1'\
        );\
    ", params![]).unwrap();

    let mut query = connection.prepare("\
        SELECT * FROM test_data;\
    ").unwrap();

    let rows = query.query_map(params![], |row| Ok(Abcd {
        alpha: row.get(0).unwrap(),
        number: row.get(1).unwrap(),
        word: row.get(2).unwrap(),
        kind: row.get(3).unwrap(),
    })).unwrap();

    let data = rows
        .map(|r| r.unwrap())
        .collect::<Vec<Abcd>>();

    assert_eq!(data, vec![
        Abcd { alpha: "A".to_string(), number: 10.0, word: "ten".to_string(), kind: "even".to_string() },
        Abcd { alpha: "B".to_string(), number: 11.0, word: "eleven".to_string(), kind: "odd".to_string() },
        Abcd { alpha: "C".to_string(), number: 12.0, word: "twelve".to_string(), kind: "even".to_string() },
        Abcd { alpha: "D".to_string(), number: 13.0, word: "thirteen".to_string(), kind: "odd".to_string() },
        Abcd { alpha: "E".to_string(), number: 14.0, word: "fourteen".to_string(), kind: "even".to_string() },
        Abcd { alpha: "F".to_string(), number: 15.0, word: "fifteen".to_string(), kind: "odd".to_string() },
    ]);
}

#[test]
fn test_abcdef_file_with_range() {
    let connection = init_connection();
    connection.execute("\
        CREATE VIRTUAL TABLE test_data USING xlite(\
            FILENAME './tests/abcdef.xlsx',\
            WORKSHEET 'Sheet1',\
            RANGE 'B2:C5'
        );\
    ", params![]).unwrap();

    let mut query = connection.prepare("\
        SELECT B, C FROM test_data;\
    ").unwrap();

    let rows = query.query_map(params![], |row| Ok(Bc {
        number: row.get(0).unwrap(),
        word: row.get(1).unwrap(),
    })).unwrap();

    let data = rows
        .map(|r| r.unwrap())
        .collect::<Vec<Bc>>();

    assert_eq!(data, vec![
        Bc { number: 11.0, word: "eleven".to_string() },
        Bc { number: 12.0, word: "twelve".to_string() },
        Bc { number: 13.0, word: "thirteen".to_string() },
        Bc { number: 14.0, word: "fourteen".to_string() },
    ]);
}

#[test]
fn test_abcdef_file_aggregate() {
    let connection = init_connection();
    connection.execute("\
        CREATE VIRTUAL TABLE test_data USING xlite(\
            FILENAME './tests/abcdef.xlsx',\
            WORKSHEET 'Sheet1'\
        );\
    ", params![]).unwrap();

    let mut query = connection.prepare("\
        SELECT D, count(*) FROM test_data GROUP BY D ORDER BY D;\
    ").unwrap();

    let rows = query.query_map(params![], |row| Ok(AbcdAgg {
        kind: row.get(0).unwrap(),
        count: row.get(1).unwrap(),
    })).unwrap();

    let data = rows
        .map(|r| r.unwrap())
        .collect::<Vec<AbcdAgg>>();

    assert_eq!(data, vec![
        AbcdAgg { kind: "even".to_string(), count: 3 },
        AbcdAgg { kind: "odd".to_string(), count: 3 },
    ]);
}

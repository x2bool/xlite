#![allow(non_upper_case_globals)]
#![allow(dead_code)]

mod options;
mod spreadsheet;
pub(crate) mod sqlite;
mod utils;

use calamine::DataType;
use std::ffi::c_void;
use std::os::raw::{c_char, c_int, c_longlong};
use std::sync::{Arc, Mutex};

use crate::options::{parse_option, UsingOption};
use crate::spreadsheet::{
    manager::{DataManager, DataManagerBuilder, DataManagerError},
    reader::DataReader,
};
use crate::sqlite::{
    sqlite3, sqlite3_api_routines, sqlite3_context, sqlite3_index_info, sqlite3_int64,
    sqlite3_module, sqlite3_value, sqlite3_vtab, sqlite3_vtab_cursor, SQLITE_ERROR, SQLITE_OK,
    SQLITE_OK_LOAD_PERMANENTLY,
};
use crate::utils::{
    collect_options_from_args, declare_table, error_to_sqlite3_string, yield_result,
};

#[no_mangle]
static mut sqlite3_api: *mut sqlite3_api_routines = std::ptr::null_mut();

#[repr(C)]
pub struct Module {
    // must be at the beginning
    base: sqlite3_module,
    name: &'static [u8],
}

#[repr(C)]
pub struct VirtualTable {
    // must be at the beginning
    base: sqlite3_vtab,
    manager: Arc<Mutex<DataManager>>,
}

#[repr(C)]
struct VirtualCursor {
    // must be at the beginning
    base: sqlite3_vtab_cursor,
    reader: Arc<Mutex<DataReader>>,
}

pub const XLITE_MODULE: Module = Module {
    base: sqlite3_module {
        iVersion: 0,
        xCreate: Some(x_create),
        xConnect: Some(x_connect),
        xBestIndex: Some(x_best_index),
        xDisconnect: Some(x_disconnect),
        xDestroy: Some(x_destroy),
        xOpen: Some(x_open),
        xClose: Some(x_close),
        xFilter: Some(x_filter),
        xNext: Some(x_next),
        xEof: Some(x_eof),
        xColumn: Some(x_column),
        xRowid: Some(x_rowid),
        xUpdate: None,
        xBegin: None,
        xSync: None,
        xCommit: None,
        xRollback: None,
        xFindFunction: None,
        xRename: None,
        xSavepoint: None,
        xRelease: None,
        xRollbackTo: None,
        xShadowName: None,
    },
    name: b"xlite\0",
};

#[no_mangle]
pub unsafe extern "C" fn register_module(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    let name = XLITE_MODULE.name;

    let result = ((*p_api).create_module.unwrap())(
        db,
        name.as_ptr() as *const c_char,
        &XLITE_MODULE as *const Module as *const sqlite3_module,
        std::ptr::null_mut(),
    );

    if result != SQLITE_OK {
        let err = format!("Failed to create module, status: {}", result);
        if let Some(ptr) = error_to_sqlite3_string(sqlite3_api, err) {
            *pz_err_msg = ptr;
        }
        SQLITE_ERROR
    } else {
        SQLITE_OK_LOAD_PERMANENTLY
    }
}

#[no_mangle]
pub unsafe extern "C" fn sqlite3_xlite_init(
    db: *mut sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut sqlite3_api_routines,
) -> c_int {
    sqlite3_api = p_api;

    let result = register_module(db, pz_err_msg, p_api);
    if result != SQLITE_OK {
        return result;
    } else {
        let result = ((*p_api).auto_extension.unwrap())(Some(std::mem::transmute(
            register_module as *const (),
        )));
        if result != SQLITE_OK {
            return result;
        }
    }

    SQLITE_OK_LOAD_PERMANENTLY
}

#[no_mangle]
unsafe extern "C" fn x_create(
    db: *mut sqlite3,
    _p_aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    let options = collect_options_from_args(argc, argv);
    let manager = DataManagerBuilder::from_options(options).open();

    let mut result: c_int = SQLITE_ERROR;

    match manager {
        Ok(mut manager) => {
            let columns = manager.get_columns();
            result = declare_table(db, sqlite3_api, columns);

            let p_new: Box<VirtualTable> = Box::new(VirtualTable {
                base: sqlite3_vtab {
                    pModule: std::ptr::null_mut(),
                    nRef: 0,
                    zErrMsg: std::ptr::null_mut(),
                },
                manager: Arc::new(Mutex::new(manager)),
            });
            *pp_vtab = Box::into_raw(p_new) as *mut sqlite3_vtab;
        }
        Err(err) => {
            let msg = match err {
                DataManagerError::NoFilename => "Filename is not provided".to_string(),
                DataManagerError::NoWorksheet => "Worksheet is not provided".to_string(),
                DataManagerError::Calamine(e) => format!("{}", e),
            };
            if let Some(ptr) = error_to_sqlite3_string(sqlite3_api, msg) {
                *pz_err = ptr;
                return SQLITE_ERROR;
            }
        }
    }

    result
}

#[no_mangle]
unsafe extern "C" fn x_connect(
    db: *mut sqlite3,
    p_aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    x_create(db, p_aux, argc, argv, pp_vtab, pz_err)
}

#[no_mangle]
unsafe extern "C" fn x_best_index(
    _p_vtab: *mut sqlite3_vtab,
    _arg1: *mut sqlite3_index_info,
) -> c_int {
    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_disconnect(p_vtab: *mut sqlite3_vtab) -> c_int {
    x_destroy(p_vtab)
}

#[no_mangle]
unsafe extern "C" fn x_destroy(p_vtab: *mut sqlite3_vtab) -> c_int {
    if !p_vtab.is_null() {
        let table = Box::from_raw(p_vtab as *mut VirtualTable);
        drop(table);
    }

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_open(
    p_vtab: *mut sqlite3_vtab,
    pp_cursor: *mut *mut sqlite3_vtab_cursor,
) -> c_int {
    let table = &mut *(p_vtab as *mut VirtualTable);
    let manager = Arc::clone(&table.manager);
    let mut lock = manager.lock().unwrap();
    let reader = lock.read();

    let cursor: Box<VirtualCursor> = Box::new(VirtualCursor {
        base: sqlite3_vtab_cursor { pVtab: p_vtab },
        reader: Arc::new(Mutex::new(reader)),
    });
    *pp_cursor = Box::into_raw(cursor) as _;

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_close(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    if !p_cursor.is_null() {
        let cursor = Box::from_raw(p_cursor as *mut VirtualCursor);
        drop(cursor);
    }

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_filter(
    _arg1: *mut sqlite3_vtab_cursor,
    _idx_num: c_int,
    _idx_str: *const c_char,
    _argc: c_int,
    _argv: *mut *mut sqlite3_value,
) -> c_int {
    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_next(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let mut reader = lock.lock().unwrap();

    reader.move_next();

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_eof(p_cursor: *mut sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    if reader.has_value() {
        0
    } else {
        1
    }
}

#[no_mangle]
unsafe extern "C" fn x_column(
    p_cursor: *mut sqlite3_vtab_cursor,
    p_context: *mut sqlite3_context,
    column: c_int,
) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    let value = reader.get_value(column as usize);
    yield_result(
        p_context,
        sqlite3_api,
        match value {
            Some(data) => data,
            None => &DataType::Empty,
        },
    );

    SQLITE_OK
}

#[no_mangle]
unsafe extern "C" fn x_rowid(
    p_cursor: *mut sqlite3_vtab_cursor,
    p_rowid: *mut sqlite3_int64,
) -> c_int {
    let cursor = &mut *(p_cursor as *mut VirtualCursor);
    let lock = Arc::clone(&cursor.reader);
    let reader = lock.lock().unwrap();

    *p_rowid = reader.get_rowid() as c_longlong;

    SQLITE_OK
}

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int, c_longlong};
use std::ptr::copy_nonoverlapping;
use calamine::DataType;
use crate::{parse_option, sqlite3, sqlite3_api_routines, sqlite3_context, UsingOption};

pub unsafe fn read_string_from_raw(raw: *const c_char) -> String {
    let cstr = CStr::from_ptr(raw);
    cstr.to_str().unwrap_or_default().to_string()
}

pub unsafe fn collect_strings_from_raw(n: usize, args: *const *const c_char) -> Vec<String> {
    let mut vec = Vec::with_capacity(n);

    let args = args as *mut *const c_char;
    for i in 0..n {
        let arg = *(args.offset(i as isize));
        let s = read_string_from_raw(arg);
        vec.push(s);
    }

    vec
}

pub unsafe fn error_to_sqlite3_string(api: *mut sqlite3_api_routines, err: String) -> Option<*mut c_char> {
    let cstr = CString::new(err).ok()?;
    let len = cstr.as_bytes_with_nul().len();

    let ptr = ((*api).malloc.unwrap())(len as c_int) as *mut c_char;
    if !ptr.is_null() {
        copy_nonoverlapping(cstr.as_ptr(), ptr, len);
        Some(ptr)
    } else {
        None
    }
}

pub unsafe fn declare_table(db: *mut sqlite3, api: *mut sqlite3_api_routines, columns: Vec<String>) -> c_int {
    let mut sql = String::from("CREATE TABLE sheet(");
    for column in columns {
        sql.push_str(column.as_str());
        sql.push(',');
    }
    sql.pop();
    sql.push(')');
    sql.push('\0');

    ((*api).declare_vtab.unwrap())(db, sql.as_bytes().as_ptr() as _)
}

pub unsafe fn collect_options_from_args(argc: c_int, argv: *const *const c_char) -> Vec<UsingOption> {
    let mut options = vec![];

    let args = collect_strings_from_raw(argc as usize, argv);
    for arg in args {
        match parse_option(arg.as_str()) {
            Ok((_, option)) => options.push(option),
            _ => {}
        }
    }

    options
}

pub unsafe fn yield_result(p_context: *mut sqlite3_context, api: *mut sqlite3_api_routines, value: &DataType) {
    match value {
        DataType::String(s) => {
            let cstr = CString::new(s.as_bytes()).unwrap();
            let len = cstr.as_bytes().len();
            let raw = cstr.into_raw();

            unsafe extern "C" fn destructor(raw: *mut c_void) {
                drop(CString::from_raw(raw as *mut c_char));
            }

            ((*api).result_text.unwrap())(
                p_context,
                raw,
                len as c_int,
                Some(destructor),
            );
        }
        DataType::Int(n) => ((*api).result_int64.unwrap())(p_context, *n as c_longlong),
        DataType::Float(f) => ((*api).result_double.unwrap())(p_context, *f),
        DataType::DateTime(f) => ((*api).result_double.unwrap())(p_context, *f),
        DataType::Bool(b) => {
            ((*api).result_int.unwrap())(p_context, if *b { 1 } else { 0 })
        }
        DataType::Empty => ((*api).result_null.unwrap())(p_context),
        DataType::Error(_e) => ((*api).result_null.unwrap())(p_context),
    }
}

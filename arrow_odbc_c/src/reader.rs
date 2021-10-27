use std::{
    ptr::{null_mut, NonNull},
    slice, str,
};

use arrow_odbc::{OdbcReader, odbc_api::{CursorImpl, handles::StatementImpl}};

use crate::{try_odbc, ArrowOdbcError, OdbcConnection};

pub struct ArrowOdbcReader(OdbcReader<CursorImpl<StatementImpl<'static>>>);

/// Creates an Arrow ODBC reader instance
///
/// # Safety
///
/// * `connection` must point to a valid OdbcConnection.
/// * `query_buf` must point to a valid utf-8 string
/// * `query_len` describes the len of `query_buf` in bytes.
#[no_mangle]
pub unsafe extern "C" fn arrow_odbc_reader_make(
    connection: NonNull<OdbcConnection>,
    query_buf: *const u8,
    query_len: usize,
    batch_size: usize,
    error_out: *mut *mut ArrowOdbcError,
) -> *mut ArrowOdbcReader {
    let query = slice::from_raw_parts(query_buf, query_len);
    let query = str::from_utf8(query).unwrap();

    let maybe_cursor = try_odbc!(connection.as_ref().0.execute(query, ()), error_out);
    if let Some(cursor) = maybe_cursor {
        let reader = try_odbc!(OdbcReader::new(cursor, batch_size), error_out);
        Box::into_raw(Box::new(ArrowOdbcReader(reader)))
    } else {
        null_mut()
    }
}

/// Frees the resources associated with an ArrowOdbcReader
///
/// # Safety
///
/// `connection` must point to a valid ArrowOdbcReader.
#[no_mangle]
pub unsafe extern "C" fn arrow_odbc_reader_free(connection: NonNull<ArrowOdbcReader>) {
    Box::from_raw(connection.as_ptr());
}

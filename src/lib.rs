//! Interface to [SQLite][1].
//!
//! ## Example
//!
//! ```
//! let connection = sqlite::open(":memory:").unwrap();
//!
//! connection.execute("
//!     CREATE TABLE `users` (id INTEGER, name VARCHAR(255));
//!     INSERT INTO `users` (id, name) VALUES (1, 'Alice');
//! ").unwrap();
//!
//! connection.process("SELECT * FROM `users`;", |pairs| {
//!     for &(column, value) in pairs.iter() {
//!         println!("{} = {}", column, value.unwrap());
//!     }
//!     true
//! }).unwrap();
//!
//! let mut statement = connection.prepare("SELECT * FROM `users`;").unwrap();
//! while let sqlite::State::Row = statement.step().unwrap() {
//!     println!("id = {}", statement.read::<i64>(0).unwrap());
//!     println!("name = {}", statement.read::<String>(1).unwrap());
//! }
//! ```
//!
//! [1]: https://www.sqlite.org

extern crate libc;
extern crate sqlite3_sys as ffi;

#[cfg(test)]
extern crate temporary;

macro_rules! raise(
    ($message:expr) => (return Err(::Error::from($message)));
);

macro_rules! error(
    ($connection:expr, $code:expr) => (
        match ::error::last($connection) {
            Some(error) => return Err(error),
            None => return Err(::Error::from(::ErrorKind::from($code as isize))),
        }
    );
);

macro_rules! ok(
    ($connection:expr, $result:expr) => (
        match $result {
            ::ffi::SQLITE_OK => {},
            code => error!($connection, code),
        }
    );
    ($result:expr) => (
        match $result {
            ::ffi::SQLITE_OK => {},
            code => return Err(::Error::from(::ErrorKind::from(code as isize))),
        }
    );
);

macro_rules! path_to_c_str(
    ($path:expr) => ({
        match $path.to_str() {
            Some(path) => match ::std::ffi::CString::new(path) {
                Ok(string) => string.as_ptr(),
                Err(_) => raise!("failed to process a path"),
            },
            None => raise!("failed to process a path"),
        }
    });
);

macro_rules! c_str_to_str(
    ($string:expr) => (::std::str::from_utf8(::std::ffi::CStr::from_ptr($string).to_bytes()));
);

macro_rules! c_str_to_string(
    ($string:expr) => (
        String::from_utf8_lossy(::std::ffi::CStr::from_ptr($string as *const _).to_bytes())
               .into_owned()
    );
);

macro_rules! str_to_c_str(
    ($string:expr) => (
        match ::std::ffi::CString::new($string) {
            Ok(string) => string.as_ptr(),
            Err(_) => raise!("failed to process a string"),
        }
    );
);

mod connection;
mod error;
mod statement;

pub use connection::Connection;
pub use error::{Error, ErrorKind};
pub use statement::{Statement, State, Parameter, Value};

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

/// Open a connection to a new or existing database.
#[inline]
pub fn open<'l, T: AsRef<std::path::Path>>(path: T) -> Result<Connection<'l>> {
    Connection::open(path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use temporary::Directory;

    pub fn setup() -> (PathBuf, Directory) {
        let directory = Directory::new("sqlite").unwrap();
        (directory.path().join("database.sqlite3"), directory)
    }
}

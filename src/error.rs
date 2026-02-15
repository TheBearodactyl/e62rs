//! error handling stuff
use thiserror::Error;

#[derive(Debug, Error)]
/// An error
pub enum E6Error {
    /// an IO error
    #[error("i/o error: {0}")]
    IO(#[from] std::io::Error),

    /// an infallible conversion error
    #[error("infallible conversion error: {0}")]
    InfallibleConversion(#[from] std::convert::Infallible),

    /// a boundbook error
    #[error("boundbook error: {0}")]
    BoundBook(#[from] boundbook::BbfError),

    /// a csv error
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),

    /// a redb transaction error
    #[error("redb transaction error: {0}")]
    RedbTransaction(#[from] redb::TransactionError),

    /// a redb table error
    #[error("redb table error: {0}")]
    RedbTable(#[from] redb::TableError),

    /// a redb storage error
    #[error("redb storage error: {0}")]
    RedbStorage(#[from] redb::StorageError),

    /// a redb commit error
    #[error("redb commit error: {0}")]
    RedbCommit(#[from] redb::CommitError),

    /// a system time error
    #[error("system time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    /// a report from color_eyre
    #[error("{0}")]
    EyreReport(#[from] color_eyre::Report),

    /// a report from miette
    #[error("{0}")]
    MietteReport(miette::Report),

    /// a reqwest error
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// a qr code error
    #[error("qr code error: {0}")]
    QR(#[from] qrcode::types::QrError),

    /// a json error
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// a toml serialization error
    #[error("toml serialization error: {0}")]
    TOMLSer(#[from] toml::ser::Error),

    /// a miette hook install error
    #[error("error installing miette hook: {0}")]
    MietteInstall(#[from] miette::InstallError),

    /// a tokio acquire error
    #[error("tokio acquire error: {0}")]
    TokioAcquire(#[from] tokio::sync::AcquireError),

    /// an int parse error
    #[error("int parse error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    /// an addr parse error
    #[error("error parsing address: {0}")]
    ParseAddr(#[from] std::net::AddrParseError),

    /// a custom error
    #[error("error: {0}")]
    Other(String),
}

impl From<String> for E6Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<miette::Report> for E6Error {
    fn from(value: miette::Report) -> Self {
        Self::MietteReport(value)
    }
}

/// A result using [`E6Error`] as the `Err` variant
pub type Result<T, U = E6Error> = miette::Result<T, U>;

/// a report
pub struct Report;

impl Report {
    /// make a new error from a compatible type
    #[allow(clippy::new_ret_no_self)]
    pub fn new<T>(e: T) -> E6Error
    where
        E6Error: std::convert::From<T>,
        T: Into<E6Error>,
    {
        E6Error::from(e)
    }
}

/// bail
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::error::E6Error::from(String::from($msg)))
    };

    ($err:expr $(,)?) => {
        return Err($crate::error::E6Error::from($err))
    };

    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::E6Error::from(format!($fmt, $($arg)*)))
    };
}

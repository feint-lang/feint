pub use result::{ScanError, ScanErrorType};
pub use scanner::{scan, scan_file, scan_optimistic};
pub use token::{Token, TokenWithLocation};

mod keyword;
mod operator;
mod result;
mod scanner;
mod token;

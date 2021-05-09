pub use result::{ScanError, ScanErrorType, ScanResult};
pub use scanner::{scan, Scanner};
pub use token::{Location, Token, TokenWithLocation};

mod keyword;
mod operator;
mod result;
mod scanner;
mod token;

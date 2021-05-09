use super::TokenWithLocation;

pub type ScanResult = Result<TokenWithLocation, ScanError>;

pub enum ScanError {
    Generic,
}

use std::fmt;

#[derive(Debug)]
pub enum WindowError {
    InvalidSelectionFormat,
    InvalidPositionSortFormat,
    InvalidRange,
    InvalidIndex,
    InvalidSortOrder,
    WindowsApiError(u32),
    Other(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::InvalidSelectionFormat => 
                write!(f, "Invalid selection format. Use 'all', '1,2,3', or '1-3'"),
            WindowError::InvalidPositionSortFormat => 
                write!(f, "Invalid position sort format. Use 'x1', 'y-1', or 'x1|y1'"),
            WindowError::InvalidRange => 
                write!(f, "Invalid range format"),
            WindowError::InvalidIndex => 
                write!(f, "Invalid index"),
            WindowError::InvalidSortOrder => 
                write!(f, "Sort order must be 1 (ascending) or -1 (descending)"),
            WindowError::WindowsApiError(code) => 
                write!(f, "Windows API error: 0x{:08x}", code),
            WindowError::Other(msg) => 
                write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for WindowError {}

pub type Result<T> = std::result::Result<T, WindowError>;
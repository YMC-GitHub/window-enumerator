use std::fmt;

/// Errors that can occur during window enumeration and inspection operations.
#[derive(Debug)]
pub enum WindowError {
    /// The selection string format is invalid.
    ///
    /// Valid formats are: "all", "1,2,3", "1-3"
    InvalidSelectionFormat,

    /// The position sort string format is invalid.
    ///
    /// Valid formats are: "x1", "y-1", "x1|y1"
    InvalidPositionSortFormat,

    /// The range format is invalid.
    ///
    /// Valid range format is: "start-end" where start <= end
    InvalidRange,

    /// The index cannot be parsed as a valid usize.
    InvalidIndex,

    /// The sort order is invalid.
    ///
    /// Valid orders are: 1 (ascending) or -1 (descending)
    InvalidSortOrder,

    /// A Windows API call failed.
    ///
    /// Contains the Windows error code.
    WindowsApiError(u32),

    /// Other unspecified errors.
    Other(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::InvalidSelectionFormat => {
                write!(f, "Invalid selection format. Use 'all', '1,2,3', or '1-3'")
            }
            WindowError::InvalidPositionSortFormat => {
                write!(
                    f,
                    "Invalid position sort format. Use 'x1', 'y-1', or 'x1|y1'"
                )
            }
            WindowError::InvalidRange => write!(f, "Invalid range format"),
            WindowError::InvalidIndex => write!(f, "Invalid index"),
            WindowError::InvalidSortOrder => {
                write!(f, "Sort order must be 1 (ascending) or -1 (descending)")
            }
            WindowError::WindowsApiError(code) => write!(f, "Windows API error: 0x{:08x}", code),
            WindowError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for WindowError {}

// 只在启用 windows 特性时提供 From 转换实现
#[cfg(feature = "windows")]
impl From<windows::core::Error> for WindowError {
    fn from(error: windows::core::Error) -> Self {
        WindowError::WindowsApiError(error.code().0 as u32) // ← 添加类型转换
    }
}

/// A specialized [`Result`] type for window operations.
pub type Result<T> = std::result::Result<T, WindowError>;
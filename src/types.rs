use std::path::PathBuf;

/// Represents a window's position and dimensions on the screen.
#[derive(Debug, Clone, Copy)]
pub struct WindowPosition {
    /// The x-coordinate of the window's top-left corner in screen coordinates.
    pub x: i32,
    /// The y-coordinate of the window's top-left corner in screen coordinates.
    pub y: i32,
    /// The width of the window in pixels.
    pub width: i32,
    /// The height of the window in pixels.
    pub height: i32,
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

/// Comprehensive information about a Windows window.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// The window handle (HWND) as an isize.
    pub hwnd: isize,
    /// The process ID (PID) that owns the window.
    pub pid: u32,
    /// The window title text.
    pub title: String,
    /// The window class name.
    pub class_name: String,
    /// The name of the process executable.
    pub process_name: String,
    /// The full path to the process executable file.
    pub process_file: PathBuf,
    /// The 1-based index of this window in enumeration results.
    pub index: usize,
    /// The position and dimensions of the window.
    pub position: WindowPosition,
}

/// Criteria for filtering windows during enumeration.
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    /// Filter by exact process ID match.
    pub pid: Option<u32>,
    /// Filter by title containing the specified string (case-insensitive).
    pub title_contains: Option<String>,
    /// Filter by class name containing the specified string (case-insensitive).
    pub class_name_contains: Option<String>,
    /// Filter by process name containing the specified string (case-insensitive).
    pub process_name_contains: Option<String>,
    /// Filter by process file path containing the specified string (case-insensitive).
    pub process_file_contains: Option<String>,
}

#[cfg(feature = "selection")]
/// Selection criteria for choosing specific windows from enumeration results.
#[derive(Debug, Clone)]
pub enum Selection {
    /// Select all windows that match the filter criteria.
    All,
    /// Select windows by their 1-based indices.
    Indices(Vec<usize>),
}

#[cfg(feature = "sorting")]
/// Position-based sorting criteria for windows.
#[derive(Debug, Clone)]
pub enum PositionSort {
    /// Sort by X coordinate only.
    X(i8), // 1: ascending, -1: descending
    /// Sort by Y coordinate only.
    Y(i8), // 1: ascending, -1: descending
    /// Sort by X coordinate first, then Y coordinate.
    XY(i8, i8), // (x_order, y_order)
}

#[cfg(feature = "sorting")]
/// Criteria for sorting window enumeration results.
#[derive(Debug, Clone)]
pub struct SortCriteria {
    /// Sort by process ID (1: ascending, -1: descending, 0: no sorting).
    pub pid: i8,
    /// Sort by window title (1: ascending, -1: descending, 0: no sorting).
    pub title: i8,
    /// Sort by window position (None: no sorting, Some: position-based sorting).
    pub position: Option<PositionSort>,
}

#[cfg(feature = "sorting")]
impl Default for SortCriteria {
    fn default() -> Self {
        Self {
            pid: 0,
            title: 0,
            position: None,
        }
    }
}

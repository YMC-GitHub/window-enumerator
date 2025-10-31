use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
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

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub pid: u32,
    pub title: String,
    pub class_name: String,
    pub process_name: String,
    pub process_file: PathBuf,
    pub index: usize,
    pub position: WindowPosition,
}

#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    pub pid: Option<u32>,
    pub title_contains: Option<String>,
    pub class_name_contains: Option<String>,
    pub process_name_contains: Option<String>,
    pub process_file_contains: Option<String>,
}

#[cfg(feature = "selection")]
#[derive(Debug, Clone)]
pub enum Selection {
    All,
    Indices(Vec<usize>),
}

#[cfg(feature = "sorting")]
#[derive(Debug, Clone)]
pub enum PositionSort {
    X(i8),        // 1: 升序, -1: 降序
    Y(i8),        // 1: 升序, -1: 降序
    XY(i8, i8),   // (x_order, y_order)
}

#[cfg(feature = "sorting")]
#[derive(Debug, Clone)]
pub struct SortCriteria {
    pub pid: i8,           // 1: 升序, -1: 降序, 0: 不排序
    pub title: i8,         // 1: 升序, -1: 降序, 0: 不排序
    pub position: Option<PositionSort>, // None: 不排序, Some: 位置排序
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
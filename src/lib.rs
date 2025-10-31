//! A powerful Windows window enumeration and inspection library.
//!
//! This crate provides advanced capabilities for discovering, filtering, sorting,
//! and selecting Windows windows with various criteria.
//!
//! # Features
//!
//! - **Window Enumeration**: Discover all visible windows on the system
//! - **Advanced Filtering**: Filter windows by PID, title, class name, process name, and file path
//! - **Sorting**: Sort windows by PID, title, or position (with `sorting` feature)
//! - **Selection**: Select specific windows by index (with `selection` feature)
//!
//! # Examples
//!
//! ```no_run
//! use window_enumerator::{WindowEnumerator, FilterCriteria};
//!
//! let mut enumerator = WindowEnumerator::new();
//! enumerator.enumerate_all_windows().unwrap();
//!
//! // Find Chrome windows using filter
//! let criteria = FilterCriteria {
//!     title_contains: Some("Chrome".to_string()),
//!     ..Default::default()
//! };
//! let chrome_windows = enumerator.filter_windows(&criteria);
//! for window in chrome_windows {
//!     window.print_compact();
//! }
//!
//! // Use filtering criteria
//! let criteria = FilterCriteria {
//!     title_contains: Some("Notepad".to_string()),
//!     ..Default::default()
//! };
//! let notepad_windows = enumerator.filter_windows(&criteria);
//! ```
//!
//! # Cargo Features
//!
//! - `sorting`: Enables window sorting capabilities
//! - `selection`: Enables window selection by indices

#![warn(missing_docs)]

mod errors;
mod models;
mod types;
mod utils;

#[cfg(feature = "windows")]
mod enumerator;

pub use errors::*;
pub use models::*;
pub use types::*;

// 公开导出工具函数
#[cfg(feature = "selection")]
pub use utils::parse_selection;

#[cfg(feature = "sorting")]
pub use utils::parse_position_sort;

#[cfg(feature = "windows")]
pub use enumerator::*;

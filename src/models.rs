use crate::types::WindowInfo;

#[cfg(feature = "sorting")]
use crate::types::{PositionSort, SortCriteria};

#[cfg(feature = "sorting")]
use crate::utils::matches_criteria;

/// Extension methods for [`WindowInfo`] providing display and validation functionality.
impl WindowInfo {
    /// Prints detailed information about the window to stdout.
    ///
    /// # Examples
    /// ```
    /// # use window_enumerator::WindowInfo;
    /// # use window_enumerator::WindowPosition;
    /// # let window = WindowInfo {
    /// #     hwnd: 12345,
    /// #     pid: 1234,
    /// #     title: "Test".to_string(),
    /// #     class_name: "TestClass".to_string(),
    /// #     process_name: "test.exe".to_string(),
    /// #     process_file: std::path::PathBuf::from("test.exe"),
    /// #     index: 1,
    /// #     position: WindowPosition::default(),
    /// # };
    /// window.print();
    /// ```
    pub fn print(&self) {
        println!("Index: {}", self.index);
        println!("Window Handle: 0x{:x}", self.hwnd);
        println!("Process ID: {}", self.pid);
        println!("Title: {}", self.title);
        println!("Class Name: {}", self.class_name);
        println!("Process Name: {}", self.process_name);
        println!("Process File: {}", self.process_file.display());
        println!(
            "Position: ({}, {}) Size: {}x{}",
            self.position.x, self.position.y, self.position.width, self.position.height
        );
        println!("----------------------------------------");
    }

    /// Prints compact window information to stdout.
    ///
    /// # Examples
    /// ```
    /// # use window_enumerator::WindowInfo;
    /// # use window_enumerator::WindowPosition;
    /// # let window = WindowInfo {
    /// #     hwnd: 12345,
    /// #     pid: 1234,
    /// #     title: "Test".to_string(),
    /// #     class_name: "TestClass".to_string(),
    /// #     process_name: "test.exe".to_string(),
    /// #     process_file: std::path::PathBuf::from("test.exe"),
    /// #     index: 1,
    /// #     position: WindowPosition::default(),
    /// # };
    /// window.print_compact();
    /// ```
    pub fn print_compact(&self) {
        println!(
            "[{}] 0x{:x} (PID: {}) @ ({},{}) - {}",
            self.index, self.hwnd, self.pid, self.position.x, self.position.y, self.title
        );
    }

    /// Checks if the window handle is still valid.
    ///
    /// This verifies that the window still exists in the system.
    ///
    /// # Examples
    /// ```
    /// # use window_enumerator::WindowInfo;
    /// # use window_enumerator::WindowPosition;
    /// # let window = WindowInfo {
    /// #     hwnd: 12345,
    /// #     pid: 1234,
    /// #     title: "Test".to_string(),
    /// #     class_name: "TestClass".to_string(),
    /// #     process_name: "test.exe".to_string(),
    /// #     process_file: std::path::PathBuf::from("test.exe"),
    /// #     index: 1,
    /// #     position: WindowPosition::default(),
    /// # };
    /// let is_valid = window.is_valid();
    /// ```
    #[cfg(feature = "windows")]
    pub fn is_valid(&self) -> bool {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        unsafe { IsWindow(HWND(self.hwnd)).as_bool() }
    }
}

/// Provides window sorting functionality.
#[cfg(feature = "sorting")]
pub struct WindowSorter;

#[cfg(feature = "sorting")]
impl WindowSorter {
    /// Sorts a vector of windows according to the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `windows` - The windows to sort (modified in-place)
    /// * `sort_criteria` - The criteria to use for sorting
    pub fn sort_windows(windows: &mut [WindowInfo], sort_criteria: &SortCriteria) {
        // ← 修改参数类型为切片
        if sort_criteria.pid == 0 && sort_criteria.title == 0 && sort_criteria.position.is_none() {
            return; // No sorting criteria
        }

        windows.sort_by(|a, b| {
            let mut ordering = std::cmp::Ordering::Equal;

            // PID sorting
            if sort_criteria.pid != 0 {
                ordering = a.pid.cmp(&b.pid);
                if sort_criteria.pid < 0 {
                    ordering = ordering.reverse();
                }
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }

            // Title sorting
            if sort_criteria.title != 0 {
                ordering = a.title.to_lowercase().cmp(&b.title.to_lowercase());
                if sort_criteria.title < 0 {
                    ordering = ordering.reverse();
                }
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }

            // Position sorting
            if let Some(ref position_sort) = sort_criteria.position {
                ordering = Self::compare_positions(a, b, position_sort);
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }

            ordering
        });
    }

    /// Compares two windows based on position sorting criteria.
    fn compare_positions(
        a: &WindowInfo,
        b: &WindowInfo,
        position_sort: &PositionSort,
    ) -> std::cmp::Ordering {
        match position_sort {
            PositionSort::X(order) => {
                let ordering = a.position.x.cmp(&b.position.x);
                if *order < 0 {
                    ordering.reverse()
                } else {
                    ordering
                }
            }
            PositionSort::Y(order) => {
                let ordering = a.position.y.cmp(&b.position.y);
                if *order < 0 {
                    ordering.reverse()
                } else {
                    ordering
                }
            }
            PositionSort::XY(x_order, y_order) => {
                // Sort by X first
                let x_ordering = a.position.x.cmp(&b.position.x);
                if x_ordering != std::cmp::Ordering::Equal {
                    return if *x_order < 0 {
                        x_ordering.reverse()
                    } else {
                        x_ordering
                    };
                }

                // If X is equal, sort by Y
                let y_ordering = a.position.y.cmp(&b.position.y);
                if *y_order < 0 {
                    y_ordering.reverse()
                } else {
                    y_ordering
                }
            }
        }
    }

    /// Filters and sorts windows according to the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `windows` - The windows to filter and sort
    /// * `criteria` - The filter criteria
    /// * `sort_criteria` - The sort criteria
    ///
    /// # Returns
    ///
    /// A new vector containing the filtered and sorted windows.
    pub fn filter_and_sort_windows(
        windows: &[WindowInfo],
        criteria: &crate::types::FilterCriteria,
        sort_criteria: &SortCriteria,
    ) -> Vec<WindowInfo> {
        let mut filtered: Vec<WindowInfo> = windows
            .iter()
            .filter(|window| matches_criteria(window, criteria))
            .cloned()
            .collect();

        Self::sort_windows(&mut filtered, sort_criteria);
        filtered
    }
}

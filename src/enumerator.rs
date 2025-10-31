use std::os::windows::ffi::OsStringExt;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::errors::{Result, WindowError};
use crate::types::{FilterCriteria, WindowInfo, WindowPosition};
use crate::utils;

#[cfg(feature = "selection")]
use crate::types::Selection;

#[cfg(feature = "sorting")]
use crate::types::SortCriteria;

#[cfg(feature = "sorting")]
use crate::models::WindowSorter;

/// The main window enumeration and inspection interface.
///
/// This struct provides methods to discover, filter, and sort Windows windows
/// with various criteria. It serves as the primary entry point for the library.
pub struct WindowEnumerator {
    windows: Vec<WindowInfo>,
}

impl WindowEnumerator {
    /// Creates a new window enumerator.
    ///
    /// The enumerator starts with no windows loaded. Call [`enumerate_all_windows`]
    /// to populate it with the current system windows.
    ///
    /// # Examples
    ///
    /// ```
    /// use window_enumerator::WindowEnumerator;
    ///
    /// let enumerator = WindowEnumerator::new();
    /// ```
    ///
    /// [`enumerate_all_windows`]: WindowEnumerator::enumerate_all_windows
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    /// Enumerates all visible windows on the system.
    ///
    /// This method populates the internal window list with all currently
    /// visible, non-child windows. Each window is assigned a 1-based index.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError::WindowsApiError`] if the Windows API call fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use window_enumerator::WindowEnumerator;
    ///
    /// let mut enumerator = WindowEnumerator::new();
    /// enumerator.enumerate_all_windows().unwrap();
    /// ```
    pub fn enumerate_all_windows(&mut self) -> Result<()> {
        self.windows.clear();

        unsafe {
            EnumWindows(
                Some(Self::enum_windows_proc),
                LPARAM(self as *mut _ as isize),
            )
            .map_err(|e| Error::new(e.code(), "Failed to enumerate windows".into()))?;
        }

        // Assign 1-based indices to each window
        for (index, window) in self.windows.iter_mut().enumerate() {
            window.index = index + 1;
        }

        Ok(())
    }

    /// Windows enumeration callback function.
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let enumerator = &mut *(lparam.0 as *mut WindowEnumerator);

        // Skip invisible windows and child windows
        if IsWindowVisible(hwnd).as_bool() && GetParent(hwnd).0 == 0 {
            if let Ok(mut window_info) = enumerator.get_window_info(hwnd) {
                // Temporary index, will be reassigned later
                window_info.index = enumerator.windows.len() + 1;
                enumerator.windows.push(window_info);
            }
        }

        BOOL::from(true) // Continue enumeration
    }

    /// Gathers information about a specific window.
    fn get_window_info(&self, hwnd: HWND) -> Result<WindowInfo> {
        unsafe {
            // Get window title
            let title = Self::get_window_text(hwnd);

            // Get window class name
            let class_name = Self::get_class_name(hwnd);

            // Get process ID
            let pid = Self::get_process_id(hwnd);

            // Get process information
            let (process_name, process_file) = if pid > 0 {
                Self::get_process_info(pid).unwrap_or_default()
            } else {
                (String::new(), std::path::PathBuf::new())
            };

            // Get window position and size
            let position = Self::get_window_position(hwnd);

            Ok(WindowInfo {
                hwnd: hwnd.0,
                pid,
                title,
                class_name,
                process_name,
                process_file,
                position,
                index: 0, // Temporary value, will be set later
            })
        }
    }

    /// Retrieves the text of a window.
    unsafe fn get_window_text(hwnd: HWND) -> String {
        let mut buffer = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len > 0 {
            std::ffi::OsString::from_wide(&buffer[..len as usize])
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        }
    }

    /// Retrieves the class name of a window.
    unsafe fn get_class_name(hwnd: HWND) -> String {
        let mut buffer = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut buffer);
        if len > 0 {
            std::ffi::OsString::from_wide(&buffer[..len as usize])
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        }
    }

    /// Retrieves the process ID of a window.
    unsafe fn get_process_id(hwnd: HWND) -> u32 {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }

    /// Retrieves the position and dimensions of a window.
    unsafe fn get_window_position(hwnd: HWND) -> WindowPosition {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            WindowPosition {
                x: rect.left,
                y: rect.top,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
            }
        } else {
            WindowPosition::default()
        }
    }

    /// Retrieves process information for a given process ID.
    unsafe fn get_process_info(pid: u32) -> Result<(String, std::path::PathBuf)> {
        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;

        let mut file_buffer = [0u16; MAX_PATH as usize];
        let len = GetProcessImageFileNameW(process_handle, &mut file_buffer);

        if len > 0 {
            let full_path = std::ffi::OsString::from_wide(&file_buffer[..len as usize]);
            let path_buf = std::path::PathBuf::from(&full_path);

            // Extract just the filename
            let process_name = path_buf
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();

            CloseHandle(process_handle).ok();
            Ok((process_name, path_buf))
        } else {
            CloseHandle(process_handle).ok();
            // 使用标准库的方法获取错误代码
            let last_error = std::io::Error::last_os_error();
            Err(WindowError::WindowsApiError(
                last_error.raw_os_error().unwrap_or(0) as u32,
            ))
        }
    }

    /// Finds windows by title containing the specified string (case-insensitive).
    ///
    /// This is a convenience method for simple title-based filtering.
    ///
    /// # Arguments
    ///
    /// * `title_substring` - The substring to search for in window titles
    ///
    /// # Returns
    ///
    /// A vector containing windows whose titles contain the specified substring.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use window_enumerator::WindowEnumerator;
    ///
    /// let mut enumerator = WindowEnumerator::new();
    /// enumerator.enumerate_all_windows().unwrap();
    ///
    /// let chrome_windows = enumerator.find_by_title("Chrome");
    /// for window in chrome_windows {
    ///     window.print_compact();
    /// }
    /// ```
    pub fn find_by_title(&self, title_substring: &str) -> Vec<WindowInfo> {
        let criteria = FilterCriteria {
            title_contains: Some(title_substring.to_string()),
            ..Default::default()
        };
        self.filter_windows(&criteria)
    }

    /// Filters windows based on the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The filter criteria to apply
    ///
    /// # Returns
    ///
    /// A vector containing only the windows that match all criteria.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use window_enumerator::{WindowEnumerator, FilterCriteria};
    ///
    /// let mut enumerator = WindowEnumerator::new();
    /// enumerator.enumerate_all_windows().unwrap();
    ///
    /// let criteria = FilterCriteria {
    ///     title_contains: Some("Chrome".to_string()),
    ///     ..Default::default()
    /// };
    /// let chrome_windows = enumerator.filter_windows(&criteria);
    /// ```
    pub fn filter_windows(&self, criteria: &FilterCriteria) -> Vec<WindowInfo> {
        self.windows
            .iter()
            .filter(|window| utils::matches_criteria(window, criteria))
            .cloned()
            .collect()
    }

    /// Filters and sorts windows based on the specified criteria.
    ///
    /// Requires the `sorting` feature.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The filter criteria to apply
    /// * `sort_criteria` - The sort criteria to apply
    ///
    /// # Returns
    ///
    /// A vector containing the filtered and sorted windows.
    #[cfg(feature = "sorting")]
    pub fn filter_and_sort_windows(
        &self,
        criteria: &FilterCriteria,
        sort_criteria: &SortCriteria,
    ) -> Vec<WindowInfo> {
        WindowSorter::filter_and_sort_windows(&self.windows, criteria, sort_criteria)
    }

    /// Filters windows with selection criteria.
    ///
    /// Requires the `selection` feature.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The filter criteria to apply
    /// * `selection` - The selection criteria to apply
    ///
    /// # Returns
    ///
    /// A vector containing the selected windows that match the filter criteria.
    #[cfg(feature = "selection")]
    pub fn filter_windows_with_selection(
        &self,
        criteria: &FilterCriteria,
        selection: &Selection,
    ) -> Vec<WindowInfo> {
        let filtered = self.filter_windows(criteria);

        match selection {
            Selection::All => filtered,
            Selection::Indices(indices) => filtered
                .into_iter()
                .filter(|window| indices.contains(&window.index))
                .collect(),
        }
    }

    /// Filters, sorts, and selects windows based on the specified criteria.
    ///
    /// Requires both `sorting` and `selection` features.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The filter criteria to apply
    /// * `sort_criteria` - The sort criteria to apply
    /// * `selection` - The selection criteria to apply
    ///
    /// # Returns
    ///
    /// A vector containing the filtered, sorted, and selected windows.
    #[cfg(all(feature = "sorting", feature = "selection"))]
    pub fn filter_sort_windows_with_selection(
        &self,
        criteria: &FilterCriteria,
        sort_criteria: &SortCriteria,
        selection: &Selection,
    ) -> Vec<WindowInfo> {
        let filtered =
            WindowSorter::filter_and_sort_windows(&self.windows, criteria, sort_criteria);

        match selection {
            Selection::All => filtered,
            Selection::Indices(indices) => filtered
                .into_iter()
                .filter(|window| indices.contains(&window.index))
                .collect(),
        }
    }

    /// Returns a reference to all enumerated windows.
    ///
    /// # Returns
    ///
    /// A slice containing all windows that were enumerated.
    pub fn get_windows(&self) -> &[WindowInfo] {
        &self.windows
    }

    /// Retrieves a window by its 1-based index.
    ///
    /// # Arguments
    ///
    /// * `index` - The 1-based index of the window to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&WindowInfo)` if a window with the given index exists, `None` otherwise.
    pub fn get_window_by_index(&self, index: usize) -> Option<&WindowInfo> {
        self.windows.iter().find(|w| w.index == index)
    }

    /// Prints all enumerated windows with their indices in a formatted table.
    ///
    /// This is useful for debugging and for users to see available windows
    /// before making selections.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use window_enumerator::WindowEnumerator;
    ///
    /// let mut enumerator = WindowEnumerator::new();
    /// enumerator.enumerate_all_windows().unwrap();
    /// enumerator.print_windows_with_indices();
    /// ```
    pub fn print_windows_with_indices(&self) {
        println!("Index | Handle      | PID    | Position    | Title");
        println!("------|-------------|--------|-------------|-------------------");
        for window in &self.windows {
            println!(
                "{:5} | 0x{:08x} | {:6} | {:4},{:4}     | {}",
                window.index,
                window.hwnd,
                window.pid,
                window.position.x,
                window.position.y,
                window.title
            );
        }
    }
}

impl Default for WindowEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

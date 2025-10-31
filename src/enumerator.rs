use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::types::{WindowInfo, WindowPosition, FilterCriteria};
use crate::errors::{WindowError, Result};
use crate::utils;

#[cfg(feature = "selection")]
use crate::types::Selection;

#[cfg(feature = "sorting")]
use crate::types::SortCriteria;

#[cfg(feature = "sorting")]
use crate::models::WindowSorter;

/// 窗口枚举器
pub struct WindowEnumerator {
    windows: Vec<WindowInfo>,
}

impl WindowEnumerator {
    pub fn new() -> Self {
        Self { windows: Vec::new() }
    }

    /// 枚举所有可见窗口
    pub fn enumerate_all_windows(&mut self) -> Result<()> {
        self.windows.clear();
        
        unsafe {
            EnumWindows(Some(Self::enum_windows_proc), LPARAM(self as *mut _ as isize))
                .map_err(|e| Error::new(e.code(), "Failed to enumerate windows"))?;
        }
        
        // 为每个窗口分配索引（从1开始）
        for (index, window) in self.windows.iter_mut().enumerate() {
            window.index = index + 1;
        }
        
        Ok(())
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let enumerator = &mut *(lparam.0 as *mut WindowEnumerator);
        
        // 跳过不可见窗口和子窗口
        if IsWindowVisible(hwnd).as_bool() && GetParent(hwnd).0 == 0 {
            if let Ok(mut window_info) = enumerator.get_window_info(hwnd) {
                // 临时索引，后续会重新分配
                window_info.index = enumerator.windows.len() + 1;
                enumerator.windows.push(window_info);
            }
        }
        
        BOOL::from(true) // 继续枚举
    }

    fn get_window_info(&self, hwnd: HWND) -> Result<WindowInfo> {
        unsafe {
            // 获取窗口标题
            let title = Self::get_window_text(hwnd);
            
            // 获取窗口类名
            let class_name = Self::get_class_name(hwnd);
            
            // 获取进程ID
            let pid = Self::get_process_id(hwnd);
            
            // 获取进程信息
            let (process_name, process_file) = if pid > 0 {
                Self::get_process_info(pid).unwrap_or_default()
            } else {
                (String::new(), std::path::PathBuf::new())
            };

            // 获取窗口位置和大小
            let position = Self::get_window_position(hwnd);

            Ok(WindowInfo {
                hwnd: hwnd.0,
                pid,
                title,
                class_name,
                process_name,
                process_file,
                position,
                index: 0, // 临时值，后续会设置
            })
        }
    }

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

    unsafe fn get_process_id(hwnd: HWND) -> u32 {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }

    unsafe fn get_window_position(hwnd: HWND) -> WindowPosition {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).as_bool() {
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

    unsafe fn get_process_info(pid: u32) -> Result<(String, std::path::PathBuf)> {
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )?;

        let mut file_buffer = [0u16; MAX_PATH as usize];
        let len = GetProcessImageFileNameW(process_handle, &mut file_buffer);
        
        if len > 0 {
            let full_path = std::ffi::OsString::from_wide(&file_buffer[..len as usize]);
            let path_buf = std::path::PathBuf::from(&full_path);
            
            // 提取文件名
            let process_name = path_buf
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();

            CloseHandle(process_handle).ok();
            Ok((process_name, path_buf))
        } else {
            CloseHandle(process_handle).ok();
            Err(WindowError::WindowsApiError(GetLastError().0))
        }
    }

    /// 根据条件过滤窗口
    pub fn filter_windows(&self, criteria: &FilterCriteria) -> Vec<WindowInfo> {
        self.windows
            .iter()
            .filter(|window| utils::matches_criteria(window, criteria))
            .cloned()
            .collect()
    }

    /// 过滤并排序窗口
    #[cfg(feature = "sorting")]
    pub fn filter_and_sort_windows(&self, criteria: &FilterCriteria, sort_criteria: &SortCriteria) -> Vec<WindowInfo> {
        WindowSorter::filter_and_sort_windows(&self.windows, criteria, sort_criteria)
    }

    /// 带选择的过滤窗口
    #[cfg(feature = "selection")]
    pub fn filter_windows_with_selection(&self, criteria: &FilterCriteria, selection: &Selection) -> Vec<WindowInfo> {
        let filtered = self.filter_windows(criteria);
        
        match selection {
            Selection::All => filtered,
            Selection::Indices(indices) => {
                filtered
                    .into_iter()
                    .filter(|window| indices.contains(&window.index))
                    .collect()
            }
        }
    }

    /// 带选择和排序的过滤窗口
    #[cfg(all(feature = "sorting", feature = "selection"))]
    pub fn filter_sort_windows_with_selection(
        &self, 
        criteria: &FilterCriteria, 
        sort_criteria: &SortCriteria,
        selection: &Selection
    ) -> Vec<WindowInfo> {
        let mut filtered = WindowSorter::filter_and_sort_windows(&self.windows, criteria, sort_criteria);
        
        match selection {
            Selection::All => filtered,
            Selection::Indices(indices) => {
                filtered
                    .into_iter()
                    .filter(|window| indices.contains(&window.index))
                    .collect()
            }
        }
    }

    /// 获取所有枚举的窗口
    pub fn get_windows(&self) -> &[WindowInfo] {
        &self.windows
    }

    /// 按索引获取窗口（1-based）
    pub fn get_window_by_index(&self, index: usize) -> Option<&WindowInfo> {
        self.windows.iter().find(|w| w.index == index)
    }

    /// 打印带索引的所有窗口
    pub fn print_windows_with_indices(&self) {
        println!("Index | Handle      | PID    | Position    | Title");
        println!("------|-------------|--------|-------------|-------------------");
        for window in &self.windows {
            println!("{:5} | 0x{:08x} | {:6} | {:4},{:4}     | {}", 
                window.index, 
                window.hwnd, 
                window.pid,
                window.position.x,
                window.position.y,
                window.title);
        }
    }
}

impl Default for WindowEnumerator {
    fn default() -> Self {
        Self::new()
    }
}
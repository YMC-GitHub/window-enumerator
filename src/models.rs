use crate::types::WindowInfo;

#[cfg(feature = "sorting")]
use crate::types::{PositionSort, SortCriteria};

#[cfg(feature = "sorting")]
use crate::utils::matches_criteria;

/// 窗口信息的方法实现
impl WindowInfo {
    /// 打印窗口信息
    pub fn print(&self) {
        println!("Index: {}", self.index);
        println!("Window Handle: 0x{:x}", self.hwnd);
        println!("Process ID: {}", self.pid);
        println!("Title: {}", self.title);
        println!("Class Name: {}", self.class_name);
        println!("Process Name: {}", self.process_name);
        println!("Process File: {}", self.process_file.display());
        println!("Position: ({}, {}) Size: {}x{}", 
            self.position.x, self.position.y, self.position.width, self.position.height);
        println!("----------------------------------------");
    }

    /// 打印简洁窗口信息
    pub fn print_compact(&self) {
        println!("[{}] 0x{:x} (PID: {}) @ ({},{}) - {}", 
            self.index, self.hwnd, self.pid, self.position.x, self.position.y, self.title);
    }

    /// 检查窗口是否仍然有效
    #[cfg(feature = "windows")]
    pub fn is_valid(&self) -> bool {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        
        unsafe { IsWindow(HWND(self.hwnd)).as_bool() }
    }
}

/// 排序功能
#[cfg(feature = "sorting")]
pub struct WindowSorter;

#[cfg(feature = "sorting")]
impl WindowSorter {
    /// 对窗口进行排序
    pub fn sort_windows(windows: &mut Vec<WindowInfo>, sort_criteria: &SortCriteria) {
        if sort_criteria.pid == 0 && sort_criteria.title == 0 && sort_criteria.position.is_none() {
            return; // 没有排序条件
        }

        windows.sort_by(|a, b| {
            let mut ordering = std::cmp::Ordering::Equal;

            // PID 排序
            if sort_criteria.pid != 0 {
                ordering = a.pid.cmp(&b.pid);
                if sort_criteria.pid < 0 {
                    ordering = ordering.reverse();
                }
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }

            // 标题排序
            if sort_criteria.title != 0 {
                ordering = a.title.to_lowercase().cmp(&b.title.to_lowercase());
                if sort_criteria.title < 0 {
                    ordering = ordering.reverse();
                }
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }

            // 位置排序
            if let Some(ref position_sort) = sort_criteria.position {
                ordering = Self::compare_positions(a, b, position_sort);
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }

            ordering
        });
    }

    /// 比较窗口位置
    fn compare_positions(a: &WindowInfo, b: &WindowInfo, position_sort: &PositionSort) -> std::cmp::Ordering {
        match position_sort {
            PositionSort::X(order) => {
                let ordering = a.position.x.cmp(&b.position.x);
                if *order < 0 { ordering.reverse() } else { ordering }
            }
            PositionSort::Y(order) => {
                let ordering = a.position.y.cmp(&b.position.y);
                if *order < 0 { ordering.reverse() } else { ordering }
            }
            PositionSort::XY(x_order, y_order) => {
                // 先按X排序
                let x_ordering = a.position.x.cmp(&b.position.x);
                if x_ordering != std::cmp::Ordering::Equal {
                    return if *x_order < 0 { x_ordering.reverse() } else { x_ordering };
                }
                
                // X相同再按Y排序
                let y_ordering = a.position.y.cmp(&b.position.y);
                if *y_order < 0 { y_ordering.reverse() } else { y_ordering }
            }
        }
    }

    /// 过滤并排序窗口
    pub fn filter_and_sort_windows(
        windows: &[WindowInfo],
        criteria: &crate::types::FilterCriteria,
        sort_criteria: &SortCriteria
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
use crate::errors::{Result, WindowError};
use crate::types::WindowInfo;

#[cfg(feature = "selection")]
use crate::types::Selection;

#[cfg(feature = "sorting")]
use crate::types::{PositionSort, SortCriteria};

/// Parses a selection string into a [`Selection`] enum.
///
/// # Examples
/// ```
/// use winspector::utils::parse_selection;
///
/// let selection = parse_selection("1,2,3").unwrap();
/// let all_selection = parse_selection("all").unwrap();
/// let range_selection = parse_selection("1-3").unwrap();
/// ```
///
/// # Errors
/// Returns [`WindowError::InvalidSelectionFormat`] if the string cannot be parsed.
#[cfg(feature = "selection")]
pub fn parse_selection(selection_str: &str) -> Result<Selection> {
    let selection_str = selection_str.trim().to_lowercase();

    if selection_str == "all" {
        return Ok(Selection::All);
    }

    let mut indices = Vec::new();
    let parts: Vec<&str> = selection_str.split(',').collect();

    for part in parts {
        let part = part.trim();
        if part.contains('-') {
            // Handle ranges like "1-3"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                let start = parse_index(range_parts[0].trim())?;
                let end = parse_index(range_parts[1].trim())?;

                for i in start..=end {
                    indices.push(i);
                }
            } else {
                return Err(WindowError::InvalidRange);
            }
        } else {
            // Handle single indices
            let index = parse_index(part)?;
            indices.push(index);
        }
    }

    // Remove duplicates and sort
    indices.sort();
    indices.dedup();

    Ok(Selection::Indices(indices))
}

/// Parses a position sort string into a [`PositionSort`] enum.
///
/// # Examples
/// ```
/// use winspector::utils::parse_position_sort;
///
/// let x_sort = parse_position_sort("x1").unwrap();
/// let y_sort = parse_position_sort("y-1").unwrap();
/// let xy_sort = parse_position_sort("x1|y1").unwrap();
/// ```
///
/// # Errors
/// Returns [`WindowError::InvalidPositionSortFormat`] if the string cannot be parsed.
#[cfg(feature = "sorting")]
pub fn parse_position_sort(sort_str: &str) -> Result<Option<PositionSort>> {
    let sort_str = sort_str.trim().to_lowercase();

    if sort_str.is_empty() {
        return Ok(None);
    }

    if sort_str.contains('|') {
        // Handle "x1|y1" format
        let parts: Vec<&str> = sort_str.split('|').collect();
        if parts.len() != 2 {
            return Err(WindowError::InvalidPositionSortFormat);
        }

        let x_part = parts[0].trim();
        let y_part = parts[1].trim();

        let x_order = parse_single_position_order(x_part, 'x')?;
        let y_order = parse_single_position_order(y_part, 'y')?;

        Ok(Some(PositionSort::XY(x_order, y_order)))
    } else {
        // Handle single coordinate sorts
        if sort_str.starts_with('x') {
            let order = parse_single_position_order(&sort_str, 'x')?;
            Ok(Some(PositionSort::X(order)))
        } else if sort_str.starts_with('y') {
            let order = parse_single_position_order(&sort_str, 'y')?;
            Ok(Some(PositionSort::Y(order)))
        } else {
            Err(WindowError::InvalidPositionSortFormat)
        }
    }
}

/// Parses a single position sort order (e.g., "x1" -> 1).
#[cfg(feature = "sorting")]
fn parse_single_position_order(part: &str, expected_prefix: char) -> Result<i8> {
    if part.len() < 2 || !part.starts_with(expected_prefix) {
        return Err(WindowError::InvalidPositionSortFormat);
    }

    let order_str = &part[1..];
    match order_str {
        "1" => Ok(1),
        "-1" => Ok(-1),
        _ => Err(WindowError::InvalidSortOrder),
    }
}

/// Parses a string into a usize index.
fn parse_index(s: &str) -> Result<usize> {
    s.parse().map_err(|_| WindowError::InvalidIndex)
}

/// Checks if a window matches the given filter criteria.
///
/// # Arguments
///
/// * `window` - The window to check
/// * `criteria` - The filter criteria to match against
///
/// # Returns
///
/// `true` if the window matches all criteria, `false` otherwise.
pub fn matches_criteria(window: &WindowInfo, criteria: &crate::types::FilterCriteria) -> bool {
    // PID filter (exact match)
    if let Some(pid) = criteria.pid {
        if window.pid != pid {
            return false;
        }
    }

    // Title filter (contains, case-insensitive)
    if let Some(ref title_filter) = criteria.title_contains {
        if !title_filter.is_empty()
            && !window
                .title
                .to_lowercase()
                .contains(&title_filter.to_lowercase())
        {
            return false;
        }
    }

    // Class name filter (contains, case-insensitive)
    if let Some(ref class_filter) = criteria.class_name_contains {
        if !class_filter.is_empty()
            && !window
                .class_name
                .to_lowercase()
                .contains(&class_filter.to_lowercase())
        {
            return false;
        }
    }

    // Process name filter (contains, case-insensitive)
    if let Some(ref process_filter) = criteria.process_name_contains {
        if !process_filter.is_empty()
            && !window
                .process_name
                .to_lowercase()
                .contains(&process_filter.to_lowercase())
        {
            return false;
        }
    }

    // Process file filter (contains, case-insensitive)
    if let Some(ref file_filter) = criteria.process_file_contains {
        if !file_filter.is_empty() {
            let file_str = window.process_file.to_string_lossy().to_lowercase();
            if !file_str.contains(&file_filter.to_lowercase()) {
                return false;
            }
        }
    }

    true
}

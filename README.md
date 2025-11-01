# Window Enumerator

[![Crates.io](https://img.shields.io/crates/v/window-enumerator)](https://crates.io/crates/window-enumerator)
[![Documentation](https://docs.rs/window-enumerator/badge.svg)](https://docs.rs/window-enumerator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.60%2B-blue.svg)](https://www.rust-lang.org)

A powerful Rust library for Windows window enumeration with advanced filtering, sorting, and selection capabilities.

## Why Use Window Enumerator?
- [user case in chinese](./user-case.why.zh.md)

## Features

- **🔍 Window Enumeration** - Discover all visible windows on the system
- **🎯 Advanced Filtering** - Filter by PID, title, class name, process name, and file path
- **📊 Multi-criteria Sorting** - Sort by PID, title, or position with flexible ordering
- **🎮 Index Selection** - Select specific windows using 1-based indices or ranges
- **🛡️ Safe API** - Memory-safe wrapper around Windows API
- **⚡ Zero-cost Abstractions** - Efficient Rust implementation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
window-enumerator = "0.4"
```

Or with specific features:

```toml
[dependencies]
window-enumerator = { version = "0.4", features = ["sorting", "selection"] }
```

## Quick Start

```rust
use window_enumerator::{WindowEnumerator, FilterCriteria};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create enumerator and enumerate windows
    let mut enumerator = WindowEnumerator::new();
    enumerator.enumerate_all_windows()?;
    
    // Print all windows in a formatted table
    enumerator.print_windows_with_indices();
    
    // Find Chrome windows
    let chrome_windows = enumerator.find_by_title("Chrome");
    println!("Found {} Chrome windows:", chrome_windows.len());
    for window in chrome_windows {
        window.print_compact();
    }
    
    Ok(())
}
```

## Usage Examples

### Basic Enumeration

```rust
use window_enumerator::WindowEnumerator;

let mut enumerator = WindowEnumerator::new();
enumerator.enumerate_all_windows()?;

// Get all windows
let windows = enumerator.get_windows();
println!("Total windows: {}", windows.len());

// Access window by index (1-based)
if let Some(window) = enumerator.get_window_by_index(1) {
    println!("First window: {}", window.title);
}
```

### Advanced Filtering

```rust
use window_enumerator::{WindowEnumerator, FilterCriteria};

let mut enumerator = WindowEnumerator::new();
enumerator.enumerate_all_windows()?;

// Filter by multiple criteria
let criteria = FilterCriteria {
    pid: Some(1234),
    title_contains: Some("Notepad".to_string()),
    process_name_contains: Some("notepad.exe".to_string()),
    ..Default::default()
};

let filtered = enumerator.filter_windows(&criteria);
println!("Found {} matching windows", filtered.len());
```

### Sorting (requires `sorting` feature)

```rust
use window_enumerator::{WindowEnumerator, SortCriteria, utils};

let mut enumerator = WindowEnumerator::new();
enumerator.enumerate_all_windows()?;

// Sort by PID ascending, then title descending
let sort_criteria = SortCriteria {
    pid: 1,    // Ascending
    title: -1, // Descending
    ..Default::default()
};

let sorted = enumerator.filter_and_sort_windows(&Default::default(), &sort_criteria);

// Sort by position (X then Y coordinates)
let position_sort = utils::parse_position_sort("x1|y1")?;
let pos_sort_criteria = SortCriteria {
    position: position_sort,
    ..Default::default()
};
let position_sorted = enumerator.filter_and_sort_windows(&Default::default(), &pos_sort_criteria);
```

### Selection (requires `selection` feature)

```rust
use window_enumerator::{WindowEnumerator, utils};

let mut enumerator = WindowEnumerator::new();
enumerator.enumerate_all_windows()?;

// Select specific indices
let selection = utils::parse_selection("1,3,5")?;
let selected = enumerator.filter_windows_with_selection(&Default::default(), &selection);

// Select range
let range_selection = utils::parse_selection("1-5")?;
let range_selected = enumerator.filter_windows_with_selection(&Default::default(), &range_selection);

// Mixed selection
let mixed_selection = utils::parse_selection("1,3-5,7")?;
let mixed_selected = enumerator.filter_windows_with_selection(&Default::default(), &mixed_selection);
```

### Combined Filtering, Sorting and Selection

```rust
use window_enumerator::{WindowEnumerator, FilterCriteria, SortCriteria, utils};

let mut enumerator = WindowEnumerator::new();
enumerator.enumerate_all_windows()?;

let criteria = FilterCriteria {
    title_contains: Some("Microsoft".to_string()),
    ..Default::default()
};

let sort_criteria = SortCriteria {
    pid: 1,
    ..Default::default()
};

let selection = utils::parse_selection("1-10")?;

let results = enumerator.filter_sort_windows_with_selection(
    &criteria,
    &sort_criteria,
    &selection
);

println!("Found {} results", results.len());
```

## API Overview

### Main Types

- **`WindowEnumerator`** - Main entry point for window operations
- **`WindowInfo`** - Detailed information about a window
- **`FilterCriteria`** - Criteria for filtering windows
- **`SortCriteria`** - Criteria for sorting windows (with `sorting` feature)
- **`Selection`** - Window selection specification (with `selection` feature)

### Key Methods

- `enumerate_all_windows()` - Discovers all visible windows
- `filter_windows()` - Filters windows based on criteria
- `filter_and_sort_windows()` - Filters and sorts windows
- `filter_windows_with_selection()` - Filters and selects windows
- `print_windows_with_indices()` - Displays windows in a formatted table

### Utility Functions

- `parse_selection()` - Parses selection strings ("all", "1,2,3", "1-3")
- `parse_position_sort()` - Parses position sort strings ("x1", "y-1", "x1|y1")

## Cargo Features

- `sorting` - Enables window sorting capabilities (enabled by default)
- `selection` - Enables window selection by indices (enabled by default)

## Platform Support

⚠️ **Windows Only**

This crate is specifically designed for Windows and uses Windows-specific APIs. It will not compile on other platforms.

## Error Handling

All operations return `window_enumerator::Result<T>` which can contain various `WindowError` variants:

```rust
use window_enumerator::{WindowEnumerator, WindowError};

let mut enumerator = WindowEnumerator::new();
match enumerator.enumerate_all_windows() {
    Ok(()) => println!("Enumeration successful"),
    Err(WindowError::WindowsApiError(code)) => {
        eprintln!("Windows API error: 0x{:08x}", code);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance Notes

- Window enumeration is performed on-demand when `enumerate_all_windows()` is called
- Filtering and sorting operations work on the pre-enumerated list for efficiency
- The library uses zero-cost abstractions where possible

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues on GitHub.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

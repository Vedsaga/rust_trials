use colored::*;
use similar::{ChangeTag, TextDiff};

fn main() {
    let old = "मेंद्रण";
    let new = "दृण";

    // Create a TextDiff object using the Myers algorithm
    let diff = TextDiff::from_chars(old, new);

    // Define custom colors for each diff operation with improved visibility on black background
    let equal_fg = CustomColor::new(220, 220, 220); // Light Gray (instead of pure white for less strain)
    let equal_bg = CustomColor::new(0, 0, 0); // Maintain black background

    let delete_fg = CustomColor::new(255, 90, 90); // Bright Coral Red (more vibrant on black)
    let delete_bg = CustomColor::new(100, 20, 20); // Deep Burgundy Background

    let insert_fg = CustomColor::new(100, 255, 100); // Bright Lime Green
    let insert_bg = CustomColor::new(20, 70, 20); // Dark Forest Green Background

    // Detailed change tracking
    let mut change_details = Vec::new();

    // Process and analyze changes
    for (idx, change) in diff.iter_all_changes().enumerate() {
        match change.tag() {
            ChangeTag::Equal => {
                change_details.push(ChangeDetail {
                    index: idx,
                    value: change.value().to_string(),
                    change_type: "Equal".to_string(),
                    color: equal_fg,
                    bg_color: equal_bg,
                });
            }
            ChangeTag::Delete => {
                change_details.push(ChangeDetail {
                    index: idx,
                    value: change.value().to_string(),
                    change_type: "Deletion".to_string(),
                    color: delete_fg,
                    bg_color: delete_bg,
                });
            }
            ChangeTag::Insert => {
                change_details.push(ChangeDetail {
                    index: idx,
                    value: change.value().to_string(),
                    change_type: "Insertion".to_string(),
                    color: insert_fg,
                    bg_color: insert_bg,
                });
            }
        }
    }

    // Detailed Change Visualization
    println!("\n🔍 Detailed Change Analysis:");
    for detail in &change_details {
        println!(
            "{} at index {}: '{}' ({})",
            match detail.change_type.as_str() {
                "Equal" => "🟢 Unchanged",
                "Deletion" => "🔴 Removed",
                "Insertion" => "🟣 Added",
                _ => "❓ Unknown",
            },
            detail.index,
            detail.value.escape_debug(), // Escape special characters
            detail.change_type
        );

        
    }

    // Visualization with colors
    println!("\n🎨 Colored Diff Representation:");
    for detail in &change_details {
        print!(
            "{}",
            detail.value
                .truecolor(detail.color.r, detail.color.g, detail.color.b)
                .on_truecolor(detail.bg_color.r, detail.bg_color.g, detail.bg_color.b)
        );
    }
    println!(); // New line after colored output

    // Statistical Summary
    let stats = get_change_stats(&change_details);
    println!("\n📊 Change Statistics:");
    println!("Total Changes: {}", stats.total_changes);
    println!("Unchanged Parts: {}", stats.unchanged);
    println!("Insertions: {}", stats.insertions);
    println!("Deletions: {}", stats.deletions);
}

// Structured representation of a change
#[derive(Debug)]
struct ChangeDetail {
    index: usize,
    value: String,
    change_type: String,
    color: CustomColor,
    bg_color: CustomColor,
}

// Statistics about changes
#[derive(Debug, Default)]
struct ChangeStats {
    total_changes: usize,
    unchanged: usize,
    insertions: usize,
    deletions: usize,
}

// Compute change statistics
fn get_change_stats(changes: &[ChangeDetail]) -> ChangeStats {
    changes.iter().fold(ChangeStats::default(), |mut stats, detail| {
        stats.total_changes += 1;
        match detail.change_type.as_str() {
            "Equal" => stats.unchanged += 1,
            "Insertion" => stats.insertions += 1,
            "Deletion" => stats.deletions += 1,
            _ => {}
        }
        stats
    })
}
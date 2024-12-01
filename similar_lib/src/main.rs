use std::collections::HashMap;

use colored::*;
use similar::{ChangeTag, TextDiff};
use uuid::Uuid;

fn main() {
    // Initialize people and UUID mappings
    let mut people: HashMap<Uuid, &str> = HashMap::new();
    let id = Uuid::new_v4();
    let value = "#JohnDoe";
    people.insert(id, value);
    let raw_input = format!("{}", id);
    let old = raw_input.replace(&id.to_string(), value);
    let new: String = "hi, #John".to_string();

    // Build the current highlight index
    let current_highlight_index = build_highlight_index(&old, id, value);

    // Create a TextDiff object using the Myers algorithm
    let diff = TextDiff::from_chars(&old, &new);

    // Prepare to track changes
    let mut change_details = Vec::new();
    let mut char_diff_result = CharDiffResult {
        insertions: Vec::new(),
        deletions: Vec::new(),
        equal_matches: Vec::new(),
    };

    // Process changes from diff only once
    let mut equal_changes = Vec::new();
    for (idx, change) in diff.iter_all_changes().enumerate() {
        let value = change.value().to_string();
        match change.tag() {
            ChangeTag::Equal => {
                let change_type = if idx == change.value().as_bytes()[0] as usize {
                    CharChangeType::EqualIndex
                } else {
                    CharChangeType::EqualDifferentIndex
                };

                change_details.push(ChangeDetail {
                    index: idx,
                    value: value.clone(),
                    color: color_equal(&change_type),
                    bg_color: bg_color_equal(&change_type),
                    change_type: change_type,
                });

                equal_changes.push((idx, value.clone()));
            }
            ChangeTag::Delete => {
                char_diff_result.deletions.push(CharMetadata {
                    index: idx,
                    char: value.clone(),
                });
                change_details.push(ChangeDetail {
                    index: idx,
                    value,
                    change_type: CharChangeType::Deletion,
                    color: delete_fg(),
                    bg_color: delete_bg(),
                });
            }
            ChangeTag::Insert => {
                char_diff_result.insertions.push(CharMetadata {
                    index: idx,
                    char: value.clone(),
                });
                change_details.push(ChangeDetail {
                    index: idx,
                    value,
                    change_type: CharChangeType::Insertion,
                    color: insert_fg(),
                    bg_color: insert_bg(),
                });
            }
        }
    }

    // Process equal matches
    for (old_idx, (new_idx, char)) in equal_changes.iter().enumerate() {
        char_diff_result.equal_matches.push(EqualCharPair {
            old_info: CharMetadata {
                index: old_idx,
                char: char.clone(),
            },
            new_info: CharMetadata {
                index: *new_idx,
                char: char.clone(),
            },
        });
    }

    // Print results
    print_change_details(&change_details);
    print_colored_diff(&change_details);
    print_equal_char_ranges(&char_diff_result);

    // Update and display highlight indexes
    let updated_highlight_index = update_highlight_index(&old, &new, &current_highlight_index);
    println!("Current Highlight Index: {:?}", current_highlight_index);
    println!("Updated Highlight Index: {:?}", updated_highlight_index);

    // Print statistical summary
    let stats = calculate_stats(&change_details);
    println!("\n📊 Change Statistics:");
    println!("Total Changes: {}", stats.total_changes);
    println!("Unchanged Parts: {}", stats.unchanged);
    println!("Insertions: {}", stats.insertions);
    println!("Deletions: {}", stats.deletions);
}

// enum for change_type
#[derive(Debug)]
enum CharChangeType {
    EqualIndex,
    EqualDifferentIndex,
    Insertion,
    Deletion,
}

// Builds the highlight index for initial text
fn build_highlight_index(old: &str, id: Uuid, value: &str) -> HashMap<Uuid, Vec<usize>> {
    let mut highlight_index: HashMap<Uuid, Vec<usize>> = HashMap::new();
    let mut start = 0;
    while let Some(pos) = old[start..].find(value) {
        let actual_pos = start + pos;
        highlight_index
            .entry(id)
            .or_default()
            .extend(actual_pos..actual_pos + value.len());
        start = actual_pos + 1;
    }
    highlight_index
}

fn update_highlight_index(
    old: &str,
    new: &str,
    current_highlight_index: &HashMap<Uuid, Vec<usize>>,
) -> HashMap<Uuid, Vec<usize>> {
    let mut updated_highlight_index = HashMap::new();

    // Create a TextDiff object to track changes
    let diff = TextDiff::from_chars(old, new);

    for (uuid, indexes) in current_highlight_index {
        let mut new_indexes = Vec::new();

        // Convert the indexes to the characters they represent
        let original_chars: Vec<char> = indexes
            .iter()
            .map(|&idx| old.chars().nth(idx).unwrap())
            .collect();

        // Track the first index after any initial changes
        let mut first_match_index = None;

        // Iterate through new string to find matching characters
        for (new_idx, change) in diff.iter_all_changes().enumerate() {
            if change.tag() == ChangeTag::Equal {
                let change_char = change.value().chars().next().unwrap();

                // If this is the first char of our tracked sequence
                if first_match_index.is_none() && original_chars.first() == Some(&change_char) {
                    first_match_index = Some(new_idx);
                }

                // If we have a first match, continue tracking the sequence
                if let Some(_) = first_match_index {
                    if new_indexes.is_empty() || new_idx == new_indexes.last().unwrap() + 1 {
                        if original_chars.get(new_indexes.len()) == Some(&change_char) {
                            new_indexes.push(new_idx);
                        } else if !new_indexes.is_empty() {
                            // Sequence interrupted
                            break;
                        }
                    } else {
                        // Non-consecutive index
                        break;
                    }
                }
            }
        }

        // Only add if we found a meaningful sequence
        if !new_indexes.is_empty() {
            updated_highlight_index.insert(*uuid, new_indexes);
        }
    }

    updated_highlight_index
}

// Calculates statistics for changes
fn calculate_stats(changes: &[ChangeDetail]) -> ChangeStats {
    changes
        .iter()
        .fold(ChangeStats::default(), |mut stats, detail| {
            stats.total_changes += 1;
            match detail.change_type {
                CharChangeType::EqualDifferentIndex | CharChangeType::EqualIndex => {
                    stats.unchanged += 1
                }
                CharChangeType::Insertion => stats.insertions += 1,
                CharChangeType::Deletion => stats.deletions += 1,
            }
            stats
        })
}

// Prints detailed changes
fn print_change_details(change_details: &[ChangeDetail]) {
    println!("\n🔍 Detailed Change Analysis:");
    for detail in change_details {
        println!(
            "{} at index {}: '{}' ({:?})",
            match detail.change_type {
                CharChangeType::EqualIndex => "⚪️ Unchanged (Same Index)",
                CharChangeType::EqualDifferentIndex => "🟡 Unchanged (Different Index)",
                CharChangeType::Insertion => "🟢 Added",
                CharChangeType::Deletion => "🔴 Removed",
            },
            detail.index,
            detail.value.escape_debug(),
            detail.change_type
        );
    }
}

// Prints colored diff
fn print_colored_diff(change_details: &[ChangeDetail]) {
    println!("\n🎨 Colored Diff Representation:");
    for detail in change_details {
        print!(
            "{}",
            detail
                .value
                .truecolor(detail.color.r, detail.color.g, detail.color.b)
                .on_truecolor(detail.bg_color.r, detail.bg_color.g, detail.bg_color.b)
        );
    }
    println!();
}

// Prints ranges for equal char pairs
fn print_equal_char_ranges(char_diff_result: &CharDiffResult) {
    println!("\n🔢 Equal Char Pair Ranges:");
    for (i, pair) in char_diff_result.equal_matches.iter().enumerate() {
        println!(
            "Pair {}: Old Index: {}, New Index: {}, Char: '{}'",
            i, pair.old_info.index, pair.new_info.index, pair.old_info.char
        );
    }
}

// Color utility functions (abstracted to reduce redundancy)
fn color_equal(change_type: &CharChangeType) -> CustomColor {
    match change_type {
        CharChangeType::EqualIndex => CustomColor::new(220, 220, 220),
        CharChangeType::EqualDifferentIndex => CustomColor::new(255, 225, 0), // More vibrant yellow
        _ => CustomColor::new(0, 0, 0),
    }
}

fn bg_color_equal(change_type: &CharChangeType) -> CustomColor {
    match change_type {
        CharChangeType::EqualIndex => CustomColor::new(0, 0, 0),
        CharChangeType::EqualDifferentIndex => CustomColor::new(100, 100, 180), // Softer yellow background
        _ => CustomColor::new(0, 0, 0),
    }
}

fn delete_fg() -> CustomColor {
    CustomColor::new(255, 90, 90) // Bright Coral Red
}

fn delete_bg() -> CustomColor {
    CustomColor::new(100, 20, 20) // Deep Burgundy Background
}

fn insert_fg() -> CustomColor {
    CustomColor::new(100, 255, 100) // Bright Lime Green
}

fn insert_bg() -> CustomColor {
    CustomColor::new(20, 70, 20) // Dark Forest Green Background
}

// Definitions for required structs
#[derive(Debug)]
struct CharMetadata {
    index: usize,
    char: String,
}

#[derive(Debug)]
struct EqualCharPair {
    old_info: CharMetadata,
    new_info: CharMetadata,
}

#[derive(Debug)]
struct CharDiffResult {
    insertions: Vec<CharMetadata>,
    deletions: Vec<CharMetadata>,
    equal_matches: Vec<EqualCharPair>,
}

#[derive(Debug)]
struct ChangeDetail {
    index: usize,
    value: String,
    change_type: CharChangeType,
    color: CustomColor,
    bg_color: CustomColor,
}

#[derive(Debug, Default)]
struct ChangeStats {
    total_changes: usize,
    unchanged: usize,
    insertions: usize,
    deletions: usize,
}

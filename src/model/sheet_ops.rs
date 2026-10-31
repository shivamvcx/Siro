fn is_invisible_or_control(c: char) -> bool {
    c.is_control()
        || ('\u{200B}'..='\u{200F}').contains(&c)
        || ('\u{202A}'..='\u{202E}').contains(&c)
        || c == '\u{FEFF}'
        || c == '\u{00AD}'
}

/// Validates a candidate sheet name according to strict application rules.
///
/// Rules:
/// 1. Strips all ASCII and Unicode control characters and invisible formatting characters.
/// 2. Trims leading and trailing whitespace.
/// 3. Collapses internal runs of whitespace to a single space.
/// 4. Rejects empty names.
/// 5. Rejects names exceeding 40 characters.
/// 6. Enforces Unicode-aware case-insensitive uniqueness within `existing_names`.
///    If `current_name` is provided (e.g. during rename), the candidate may match `current_name`
///    (case change or identical name is allowed, but colliding with any *other* sheet is rejected).
pub fn validate_sheet_name(
    raw_name: &str,
    existing_names: &[&str],
    current_name: Option<&str>,
) -> Result<String, String> {
    // 1. Strip control and invisible formatting characters
    let stripped: String = raw_name.chars().filter(|c| !is_invisible_or_control(*c)).collect();

    // 2 & 3. Trim and collapse internal whitespace runs
    let collapsed: String = stripped
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    // 4. Reject empty names
    if collapsed.trim().is_empty() {
        return Err("Sheet name cannot be empty".to_string());
    }

    // 5. Reject names over 40 characters
    if collapsed.chars().count() > 40 {
        return Err("Sheet name cannot exceed 40 characters".to_string());
    }

    // 6. Case-insensitive uniqueness check (Unicode case-folded)
    let candidate_lower = collapsed.to_lowercase();
    let is_same_as_current = current_name
        .map(|cur| candidate_lower == cur.trim().to_lowercase())
        .unwrap_or(false);

    if !is_same_as_current {
        for &existing in existing_names {
            // Skip comparing against current sheet's existing name
            if let Some(cur) = current_name {
                if existing.trim().to_lowercase() == cur.trim().to_lowercase() {
                    continue;
                }
            }
            if candidate_lower == existing.trim().to_lowercase() {
                return Err(format!("A sheet named \"{}\" already exists", collapsed));
            }
        }
    }

    Ok(collapsed)
}

/// Generates a unique suggested default sheet name (e.g. "Sheet 2", "Sheet 3").
///
/// Starts checking from `max(1, existing_names.len() + 1)` and increments until a name
/// is found that does not exist in `existing_names` (case-insensitive).
pub fn suggest_next_sheet_name(existing_names: &[&str]) -> String {
    let mut candidate_num = existing_names.len() + 1;
    if candidate_num == 0 {
        candidate_num = 1;
    }

    loop {
        let candidate = format!("Sheet {}", candidate_num);
        let candidate_lower = candidate.to_lowercase();
        let exists = existing_names
            .iter()
            .any(|&name| name.trim().to_lowercase() == candidate_lower);
        if !exists {
            return candidate;
        }
        candidate_num += 1;
    }
}

/// Truncates a string to at most `max_chars`, appending an ellipsis ("…") if truncated.
/// Safely handles multibyte UTF-8 character boundaries.
pub fn truncate_with_ellipsis(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }

    let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sheet_name_normal() {
        let existing = ["Main Schematic", "Sheet 2"];
        let res = validate_sheet_name("Power Supply", &existing, None);
        assert_eq!(res.unwrap(), "Power Supply");
    }

    #[test]
    fn test_validate_sheet_name_whitespace_and_control_chars() {
        let existing = ["Main Schematic"];
        let res = validate_sheet_name("  \t Analog\x00\x07  Circuit  \n ", &existing, None);
        assert_eq!(res.unwrap(), "Analog Circuit");
    }

    #[test]
    fn test_validate_sheet_name_rejects_empty() {
        let existing = ["Main Schematic"];
        assert!(validate_sheet_name("", &existing, None).is_err());
        assert!(validate_sheet_name("   \t\n  ", &existing, None).is_err());
        assert!(validate_sheet_name("\x00\x1f", &existing, None).is_err());
    }

    #[test]
    fn test_validate_sheet_name_max_length() {
        let existing = ["Main Schematic"];
        let name_40 = "a".repeat(40);
        assert_eq!(validate_sheet_name(&name_40, &existing, None).unwrap(), name_40);

        let name_41 = "a".repeat(41);
        assert!(validate_sheet_name(&name_41, &existing, None).is_err());
    }

    #[test]
    fn test_validate_sheet_name_case_insensitive_duplicate() {
        let existing = ["Main Schematic", "Sheet 2", "Übersicht"];
        assert!(validate_sheet_name("main schematic", &existing, None).is_err());
        assert!(validate_sheet_name("SHEET 2", &existing, None).is_err());
        assert!(validate_sheet_name("  Sheet 2  ", &existing, None).is_err());
        // Unicode case folding check
        assert!(validate_sheet_name("übersicht", &existing, None).is_err());
        assert!(validate_sheet_name("\u{200B}\u{FEFF}", &existing, None).is_err(), "Zero-width spaces must be rejected as empty");
    }

    #[test]
    fn test_validate_sheet_name_rename_self_and_case_change() {
        let existing = ["Main Schematic", "Sheet 2"];
        // Renaming to identical name is allowed (no-op)
        assert_eq!(
            validate_sheet_name("Sheet 2", &existing, Some("Sheet 2")).unwrap(),
            "Sheet 2"
        );
        // Changing case only of current sheet is allowed
        assert_eq!(
            validate_sheet_name("sheet 2", &existing, Some("Sheet 2")).unwrap(),
            "sheet 2"
        );
        // Colliding with another sheet during rename is rejected
        assert!(validate_sheet_name("main schematic", &existing, Some("Sheet 2")).is_err());
    }

    #[test]
    fn test_suggest_next_sheet_name() {
        assert_eq!(suggest_next_sheet_name(&[]), "Sheet 1");
        assert_eq!(suggest_next_sheet_name(&["Main Schematic"]), "Sheet 2");
        assert_eq!(suggest_next_sheet_name(&["Main Schematic", "Sheet 2"]), "Sheet 3");
        // If Sheet 3 already exists, should advance to Sheet 4
        assert_eq!(
            suggest_next_sheet_name(&["Main Schematic", "Sheet 2", "sheet 3"]),
            "Sheet 4"
        );
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("Short", 10), "Short");
        assert_eq!(truncate_with_ellipsis("ExactTen10", 10), "ExactTen10");
        assert_eq!(truncate_with_ellipsis("LongerThanTenCharacters", 10), "LongerTha…");
        assert_eq!(truncate_with_ellipsis("🚀 Microcontroller System", 10), "🚀 Microco…");
    }
}

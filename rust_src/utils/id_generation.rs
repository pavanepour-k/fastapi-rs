use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generate unique operation ID for FastAPI routes
pub fn generate_unique_id(route_name: &str, method: &str, path: &str) -> String {
    // Convert method to lowercase for consistency
    let method_lower = method.to_lowercase();

    // Clean up route name
    let clean_name = route_name
        .replace("_", " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("");

    // If we have a clean name, use method + name pattern
    if !clean_name.is_empty() && clean_name != route_name {
        format!("{}_{}", method_lower, snake_case(&clean_name))
    } else {
        // Fallback to path-based generation
        generate_id_from_path(&method_lower, path)
    }
}

/// Generate ID from HTTP method and path
fn generate_id_from_path(method: &str, path: &str) -> String {
    let mut parts = Vec::new();
    parts.push(method.to_string());

    for segment in path.split('/') {
        if segment.is_empty() {
            continue;
        }

        if segment.starts_with('{') && segment.ends_with('}') {
            // Extract parameter name without type annotation
            let param_name = segment
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split(':')
                .next()
                .unwrap_or("param");
            parts.push(format!("by_{}", param_name));
        } else {
            parts.push(clean_path_segment(segment));
        }
    }

    parts.join("_")
}

/// Clean path segment for use in identifiers
fn clean_path_segment(segment: &str) -> String {
    segment
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

/// Convert CamelCase to snake_case
fn snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if !result.is_empty() && !result.ends_with('_') {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_lowercase() {
                        result.push('_');
                    }
                }
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }

    result
}

/// Generate UUID-like identifier
pub fn generate_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let hash1 = hasher.finish();

    (hash1 ^ 0x123456789abcdef0).hash(&mut hasher);
    let hash2 = hasher.finish();

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (hash1 >> 32) as u32,
        (hash1 >> 16) as u16,
        hash1 as u16,
        (hash2 >> 48) as u16,
        hash2 & 0xffffffffffff
    )
}

/// Generate short hash for identifiers
pub fn generate_short_hash(input: &str, length: usize) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    let charset = "abcdefghijklmnopqrstuvwxyz0123456789";
    let charset_bytes = charset.as_bytes();
    let mut result = String::with_capacity(length);

    let mut current_hash = hash;
    for _ in 0..length {
        let index = (current_hash as usize) % charset_bytes.len();
        result.push(charset_bytes[index] as char);
        current_hash = current_hash.wrapping_mul(31).wrapping_add(1);
    }

    result
}

/// Generate operation ID with collision detection
pub fn generate_operation_id_with_collision_detection(
    base_name: &str,
    existing_ids: &std::collections::HashSet<String>,
) -> String {
    let mut candidate = base_name.to_string();
    let mut counter = 1;

    while existing_ids.contains(&candidate) {
        candidate = format!("{}_{}", base_name, counter);
        counter += 1;
    }

    candidate
}

/// Generate readable identifier from text
pub fn generate_readable_id(text: &str) -> String {
    text.chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c.to_lowercase().next().unwrap())
            } else if c.is_whitespace() || c == '-' || c == '_' {
                Some('_')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Generate timestamp-based ID
pub fn generate_timestamp_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let mut hasher = DefaultHasher::new();
    timestamp.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{:x}_{:x}", timestamp, hash & 0xffffff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_unique_id() {
        // Test with clean route name
        let id = generate_unique_id("get_users", "GET", "/users");
        assert_eq!(id, "get_get_users");

        // Test with path-based generation
        let id = generate_unique_id("", "GET", "/users/{id}");
        assert_eq!(id, "get_users_by_id");

        // Test with complex path
        let id = generate_unique_id("", "POST", "/api/v1/users/{user_id}/posts");
        assert_eq!(id, "post_api_v1_users_by_user_id_posts");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(snake_case("CamelCase"), "camel_case");
        assert_eq!(snake_case("HTMLParser"), "html_parser");
        assert_eq!(snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(snake_case("simple"), "simple");
        assert_eq!(snake_case(""), "");
    }

    #[test]
    fn test_clean_path_segment() {
        assert_eq!(clean_path_segment("users"), "users");
        assert_eq!(clean_path_segment("user-profile"), "userprofile");
        assert_eq!(clean_path_segment("api@v1"), "apiv1");
        assert_eq!(clean_path_segment("123abc"), "123abc");
    }

    #[test]
    fn test_generate_uuid() {
        let uuid1 = generate_uuid();
        let uuid2 = generate_uuid();

        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.len(), 36); // Standard UUID format length
        assert!(uuid1.contains('-'));
    }

    #[test]
    fn test_generate_short_hash() {
        let hash1 = generate_short_hash("test", 8);
        let hash2 = generate_short_hash("test", 8);
        let hash3 = generate_short_hash("different", 8);

        assert_eq!(hash1.len(), 8);
        assert_eq!(hash1, hash2); // Same input should give same hash
        assert_ne!(hash1, hash3); // Different input should give different hash
    }

    #[test]
    fn test_collision_detection() {
        let mut existing = HashSet::new();
        existing.insert("test".to_string());
        existing.insert("test_1".to_string());

        let id = generate_operation_id_with_collision_detection("test", &existing);
        assert_eq!(id, "test_2");

        let id2 = generate_operation_id_with_collision_detection("unique", &existing);
        assert_eq!(id2, "unique");
    }

    #[test]
    fn test_generate_readable_id() {
        assert_eq!(generate_readable_id("Hello World"), "hello_world");
        assert_eq!(generate_readable_id("API-Endpoint_v1"), "api_endpoint_v1");
        assert_eq!(generate_readable_id("User@Profile!"), "user_profile");
        assert_eq!(generate_readable_id("123_test_456"), "123_test_456");
    }

    #[test]
    fn test_generate_timestamp_id() {
        let id1 = generate_timestamp_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = generate_timestamp_id();

        assert_ne!(id1, id2);
        assert!(id1.contains('_'));
        assert!(id1.len() > 10);
    }
}

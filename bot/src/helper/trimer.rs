pub fn trim(desc: String, length_diff: i32) -> String {
	if length_diff <= 0 {
		let mut desc_trim;
		let abs_diff = (-length_diff) as usize;

		// Handle empty string case
		if desc.is_empty() {
			return "...".to_string();
		}

		// Handle case where we would trim too much
		if desc.len() <= abs_diff + 3 {
			return "...".to_string();
		}

		let trim_length = desc.len() - (abs_diff + 3);
		desc_trim = format!("{}...", &desc[..trim_length]);

		let count = desc_trim.matches("||").count();

		if count % 2 != 0 {
			// Handle case where we would trim too much with spoiler tags
			if desc.len() <= abs_diff + 5 {
				return "||...||".to_string();
			}

			let trim_length = desc.len() - (abs_diff + 5);
			desc_trim = format!("{}||...", &desc[..trim_length])
		}

		desc_trim
	} else {
		desc
	}
}

pub fn trim_webhook(desc: String, lenght_diff: i32) -> String {
	if lenght_diff <= 0 {
		// Handle edge cases where the string is empty or the trim would result in an empty string
		let abs_diff = (-lenght_diff) as usize;
		if desc.len() <= abs_diff {
			return String::new(); // Return empty string if we would trim everything
		}

		let trim_length = desc.len() - abs_diff;
		desc[..trim_length].to_string()
	} else {
		desc
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_trim() {
		// Test case 1: Basic trimming
		let desc = String::from("Hello, world!");
		let length_diff = 6 - desc.len() as i32;
		let result = trim(desc, length_diff);
		assert_eq!(result, "Hel...");

		// Test case 2: Trimming with Discord spoiler tags
		let desc = String::from("Hello, || world! ||");
		let length_diff = 14 - desc.len() as i32;
		let result = trim(desc, length_diff);
		assert_eq!(result, "Hello, ||||...");

		// Test case 3: No trimming needed (positive length_diff)
		let desc = String::from("Hello, world!");
		let length_diff = 13;
		let result = trim(desc, length_diff);
		assert_eq!(result, "Hello, world!");

		// Test case 4: Empty string
		let desc = String::from("");
		let length_diff = -3;
		let result = trim(desc, length_diff);
		assert_eq!(result, "...");

		// Test case 5: Exactly at the limit (length_diff = 0)
		let desc = String::from("Hello, world!");
		let length_diff = 0;
		let result = trim(desc, length_diff);
		assert_eq!(result, "Hello, wor...");

		// Test case 6: Multiple spoiler tags with odd count after trimming
		let desc = String::from("Hello, || world || and || more ||");
		let length_diff = 15 - desc.len() as i32;
		let result = trim(desc, length_diff);
		// Should ensure the spoiler tags are properly balanced
		assert!(result.matches("||").count() % 2 == 0, "Spoiler tags should be balanced");
	}

	#[test]
	fn test_trim_webhook() {
		// Test case 1: Basic trimming
		let desc = String::from("Hello, world!");
		let length_diff = 3 - desc.len() as i32;
		let result = trim_webhook(desc, length_diff);
		assert_eq!(result, "Hel");

		// Test case 2: No trimming needed (positive length_diff)
		let desc = String::from("Hello, world!");
		let length_diff = 13 - desc.len() as i32;
		let result = trim_webhook(desc, length_diff);
		assert_eq!(result, "Hello, world!");

		// Test case 3: Empty string
		let desc = String::from("");
		let length_diff = -3;
		let result = trim_webhook(desc, length_diff);
		assert_eq!(result, "");

		// Test case 4: Exactly at the limit (length_diff = 0)
		let desc = String::from("Hello, world!");
		let length_diff = 0;
		let result = trim_webhook(desc, length_diff);
		assert_eq!(result, "Hello, world!");

		// Test case 5: Trim to empty string
		let desc = String::from("Hello");
		let length_diff = -5;
		let result = trim_webhook(desc, length_diff);
		assert_eq!(result, "");
	}
}

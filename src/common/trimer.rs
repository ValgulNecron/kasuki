/// This function trims a given string based on a length difference parameter.
///
/// # Arguments
///
/// * `desc` - An input string that needs to be trimmed.
/// * `lenght_diff` - The difference in the length for trimming the string. If the difference is less than or equal to 0, trimming will occur based on this difference.
///
/// # Returns
///
/// This function returns a trimmed version of the `desc` input string. If `lenght_diff` is less than or equal to 0, the string is trimmed depending on the absolute difference.
///
/// In the case when the number of substring "||" is odd in the trimmed string, it trims the original string up to an additional two characters from the end and adds "||.." to the end. The function returns the original string if `lenght_diff` is more than 0.
///
pub fn trim(desc: String, lenght_diff: i32) -> String {
    if lenght_diff <= 0 {
        let mut desc_trim;
        let trim_length = desc.len() - ((-lenght_diff) as usize + 3);
        desc_trim = format!("{}...", &desc[..trim_length]);

        let count = desc_trim.matches("||").count();
        if count % 2 != 0 {
            let trim_length = desc.len() - ((-lenght_diff) as usize + 5);
            desc_trim = format!("{}||...", &desc[..trim_length]);
        }
        desc_trim.clone()
    } else {
        desc
    }
}

/// Trims the given `String` to a specific length based on a difference value.
///
/// # Arguments
/// * `desc` - A `String` which is the original text that needs to be trimmed.
/// * `length_diff` - An `i32` which indicates how much shorter the text should be compared to its original length.
///   If it's 0 or less, the `desc` will be trimmed by the absolute value of `length_diff`.
///   If `length_diff` is more than 0, the original `desc` string will be returned. It's assumed that the `length_diff` is not greater than the length of `desc` string.
///
/// # Return
/// The function returns the trimmed `String`. If `length_diff` is equal to or less than 0, it returns the trimmed version of `desc` text.
/// If `length_diff` is more than 0, it returns the original `desc` string.
pub fn trim_webhook(desc: String, lenght_diff: i32) -> String {
    if lenght_diff <= 0 {
        let trim_length = desc.len() - (-lenght_diff) as usize;
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
        let desc = String::from("Hello, world!");
        let length_diff = 6 - desc.len() as i32;
        let result = trim(desc, length_diff);
        assert_eq!(result, "Hel...");

        let desc = String::from("Hello, || world! ||");
        let length_diff = 14 - desc.len() as i32;
        let result = trim(desc, length_diff);
        assert_eq!(result, "Hello, ||||...");

        let desc = String::from("Hello, world!");
        let length_diff = 13;
        let result = trim(desc, length_diff);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_trim_webhook() {
        let desc = String::from("Hello, world!");
        let length_diff = 3 - desc.len() as i32;
        let result = trim_webhook(desc, length_diff);
        assert_eq!(result, "Hel");

        let desc = String::from("Hello, world!");
        let length_diff = 13 - desc.len() as i32;
        let result = trim_webhook(desc, length_diff);
        assert_eq!(result, "Hello, world!");
    }
}

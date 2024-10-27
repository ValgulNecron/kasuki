pub fn trim(desc: String, length_diff: i32) -> String {
    if length_diff <= 0 {
        let mut desc_trim;

        let trim_length = desc.len() - ((-length_diff) as usize + 3);

        desc_trim = format!("{}...", &desc[..trim_length]);

        let count = desc_trim.matches("||").count();

        if count % 2 != 0 {
            let trim_length = desc.len() - ((-length_diff) as usize + 5);

            desc_trim = format!("{}||...", &desc[..trim_length])
        }

        desc_trim.clone()
    } else {
        desc
    }
}

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

        assert_eq!(result, "Hello, world!")
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

        assert_eq!(result, "Hello, world!")
    }
}

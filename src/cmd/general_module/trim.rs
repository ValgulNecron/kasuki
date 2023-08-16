///
///
/// # Arguments
///
/// * `desc`: The description that will be set on the discord embed that is 4096 or longer in size
/// * `lenght_diff`: The lenght difference between the max of 4096 and the description
///
/// returns: The trimmed description to have a 4096 lenght.
///
/// # Examples
///
/// ```
/// let lenght_diff = 4096 - desc.len() as i32;
/// if lenght_diff <= 0 {
///     desc = trim(desc, lenght_diff)
/// }
/// desc
/// ```
pub fn trim(desc: String, lenght_diff: i32) -> String {
    return if lenght_diff <= 0 {
        let mut desc_trim;
        let trim_length = desc.len() - ((lenght_diff * -1) as usize + 3);
        desc_trim = format!("{}...", &desc[..trim_length]);

        let count = desc_trim.matches("||").count();
        if count % 2 != 0 {
            let trim_length = desc.len() - ((lenght_diff * -1) as usize + 5);
            desc_trim = format!("{}||..", &desc[..trim_length]);
        }
        let trim = desc_trim.clone();
        trim
    } else {
        desc
    }
}

pub fn trim_webhook(desc: String, lenght_diff: i32) -> String {
    return if lenght_diff <= 0 {
        let desc_trim;
        let trim_length = desc.len() - (lenght_diff * -1) as usize;
        desc_trim = format!("{}", &desc[..trim_length]);
        let trim = desc_trim.clone();
        trim
    } else {
        desc
    }
}

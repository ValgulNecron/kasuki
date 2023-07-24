pub fn trim(desc: String, lenght_diff: i32) -> String {
    if lenght_diff <= 0 {
        let mut desc_trim;
        let trim_length = desc.len() - ((lenght_diff * -1) as usize + 3);
        desc_trim = format!("{}...", &desc[..trim_length]);

        let count = desc_trim.matches("||").count();
        if count % 2 != 0 {
            let trim_length = desc.len() - ((lenght_diff * -1) as usize + 5);
            desc_trim = format!("{}||..", &desc[..trim_length]);
        }
        let trim = desc_trim.clone();
        return trim;
    } else {
        return desc;
    }
}

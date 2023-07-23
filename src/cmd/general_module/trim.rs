pub fn trim(desc: String, lenght_diff: i32) -> String {
    let count = desc.matches("||").count();
    let desc_trim;
    println!("{}", count);
    if count % 2 == 0 {
        let trim_length = desc.len() - ((lenght_diff * -1) as usize + 5);
        desc_trim = format!("{}||...", &desc[..trim_length]);
    } else {
        let trim_length = desc.len() - ((lenght_diff * -1) as usize + 5);
        desc_trim = format!("{}||...", &desc[..trim_length]);
    }
    return desc_trim;
}
pub fn hash_file(path: &str) -> Vec<u8> {
    todo!()
}

pub fn sort_files(path: &str) -> Vec<(String, Vec<u8>)> {
    todo!()
}

pub fn closest_bigger_power_of_two(n: u32) -> u32 {
    let log_value = (n as f64).log2();
    let ceil_value = log_value.ceil() as u32;
    2u32.pow(ceil_value)
}

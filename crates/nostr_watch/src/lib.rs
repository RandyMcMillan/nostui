use online::online;
mod online;
pub fn run(name: &str) {
    match name {
        "online" => online(),
        _ => panic!("Invalid option"),
    };
}

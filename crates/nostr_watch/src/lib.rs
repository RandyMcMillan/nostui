use offline::offline;
mod offline;
use online::online;
mod online;
use paid::paid;
mod paid;

use nip0::nip0;
mod nip0;

pub fn run(name: &str) {
    match name {
        "offline" => offline(),
        "online" => online(),
        "paid" => paid(),
        "nip0" => nip0(),
        _ => panic!("Invalid option"),
    };
}

use offline::offline;
mod offline;
use online::online;
mod online;
use paid::paid;
mod paid;

use nip0::nip0;
mod nip0;
use nip1::nip1;
mod nip1;

pub fn run(name: &str) {
    match name {
        "offline" => offline(),
        "online" => online(),
        "paid" => paid(),
        "nip0" => nip0(),
        "nip1" => nip1(),
        _ => panic!("Invalid option"),
    };
}

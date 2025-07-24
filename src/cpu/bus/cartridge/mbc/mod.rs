use mbc1::MBC1;
use mbc3::MBC3;
use mbc5::MBC5;
use serde::{Deserialize, Serialize};

pub mod mbc1;
pub mod mbc3;
pub mod mbc5;

#[derive(Serialize, Deserialize)]
pub enum MBC {
    None,
    MBC1(MBC1),
    MBC3(MBC3),
    MBC5(MBC5)
}
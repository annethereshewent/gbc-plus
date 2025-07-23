use bitflags::bitflags;
use serde::{Deserialize, Serialize};


bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct SoundPanningRegister: u8 {
        const CH1_LEFT = 1;
        const CH1_RIGHT = 1 << 1;
        const CH2_LEFT = 1 << 2;
        const CH2_RIGHT = 1 << 3;
        const CH3_LEFT = 1 << 4;
        const CH3_RIGHT = 1 << 5;
        const CH4_LEFT = 1 << 6;
        const CH4_RIGHT = 1 << 7;
    }
}
//! Minimal Rust client for the Nightscout v3 REST API.
//!
//! Docs: <https://github.com/nightscout/cgm-remote-monitor/blob/master/lib/api3/swagger.yaml>

pub mod client;
pub mod models;

pub use client::NightscoutClient;
pub use models::{
    Devicestatus, DocumentBase, Entry, Food, LoopCob, LoopIob, LoopStatus, NightscoutBearer,
    NightscoutSecrets, Profile, ProfileStore, PumpBattery, PumpStatus, TimeValue, Treatment,
};

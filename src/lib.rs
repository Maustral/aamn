pub mod crypto;
pub mod protocol;
pub mod tunnel;
pub mod network;
pub mod routing;
pub mod transport;
pub mod fragment;
pub mod circuit;
pub mod pow;
pub mod handshake;
pub mod rate_limiter;
pub mod metrics;
pub mod config;
pub mod cli;
pub mod logging;
pub mod daemon;
pub mod error;
pub mod dht;
pub mod padding;
pub mod fuzzing;
pub mod integration_tests;


pub const PROTOCOL_VERSION: u8 = 1;
pub const MAX_PACKET_SIZE: usize = 1450;

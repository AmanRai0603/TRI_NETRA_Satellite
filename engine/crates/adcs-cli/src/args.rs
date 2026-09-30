//! A flying command (run, params, parity, size) as the engine takes it; cli.rs builds it
//! from the parsed command line.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use adcs_fsw_abi::Impl;
use adcs_sim::run;
use std::path::PathBuf;

pub struct Args {
    pub cmd: String, pub scenario: String, pub case: Option<String>, pub fsw: Impl, pub fsw_b: Option<Impl>, pub seed: u64,
    pub out: Option<PathBuf>, pub set: Vec<(String, String)>, pub quiet: bool, pub realtime: bool, pub oils: Option<run::OilsModel>,
}

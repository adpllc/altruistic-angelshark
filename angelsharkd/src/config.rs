use anyhow::{Context, Result};
use libangelshark::{Acm, AcmRunner};
use std::{
    env,
    fs::File,
    net::{Ipv4Addr, SocketAddrV4},
};

#[derive(Clone)]
pub struct Config {
    pub bind_addr: SocketAddrV4,
    pub debug_mode: bool,
    pub runner: AcmRunner,
    pub origin: String,
}

impl Config {
    pub fn init() -> Result<Self> {
        let debug_mode = cfg!(debug_assertions) || env::var_os("ANGELSHARKD_DEBUG").is_some();

        let bind_addr: SocketAddrV4 = env::var("ANGELSHARKD_ADDR")
            .map(|addr| addr.parse())
            .unwrap_or_else(|_| Ok(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)))
            .with_context(|| "Failed to parse socket bind address.")?;

        let origin = env::var("ANGELSHARKD_ORIGIN").unwrap_or_default();

        let logins = if let Ok(path) = env::var("ANGELSHARKD_LOGINS") {
            File::open(path)
        } else {
            File::open("./asa.cfg")
        }
        .with_context(|| "Failed to open logins file.")?;

        let mut runner = AcmRunner::default();
        for (job_name, acm) in
            Acm::from_logins(logins).with_context(|| "Failed to parse logins.")?
        {
            runner.register_acm(&job_name, acm);
        }

        Ok(Self {
            bind_addr,
            origin,
            debug_mode,
            runner,
        })
    }
}

use crate::Message;
use anyhow::{anyhow, Context, Result};
use cached::{proc_macro::cached, Return, TimedCache};
use log::info;
use ssh2::{KeyboardInteractivePrompt, Prompt, Session, Stream};
use std::{
    fmt::Debug,
    io::{BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, TcpStream},
};

const DEFAULT_PORT: u16 = 5022;
const OSSI_LOGOFF: &[u8] = b"clogoff\nt\ny\n";
const OSSI_MAN_TERM: &[u8] = b"ossiem\n";
const OSSI_TERM: &[u8] = b"ossie\n";
const TERM: &str = "vt100";
const TERM_DIMS: (u32, u32, u32, u32) = (81, 25, 0, 0);
const TIMEOUT_MS: u32 = 30000; // Thirty second read/write timeout.

/// Represents a Communication Manager and its login information. Executes
/// collections of [Message]s on an ACM over SSH.
#[derive(Clone)]
pub struct Acm {
    addr: Ipv4Addr,
    port: Option<u16>,
    user: String,
    pass: String,
}

impl Debug for Acm {
    /// Formats an ACM's configuration information for debugging. Masks passwords.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acm")
            .field("addr", &self.addr)
            .field("port", &self.port.unwrap_or(DEFAULT_PORT))
            .field("user", &self.user)
            .field("pass", &"********")
            .finish()
    }
}

impl Default for Acm {
    fn default() -> Self {
        Self {
            addr: Ipv4Addr::new(127, 0, 0, 1),
            port: Default::default(),
            user: Default::default(),
            pass: Default::default(),
        }
    }
}

impl Acm {
    /// Adds an IPv4 address to ACM config.
    pub fn with_addr(&mut self, addr: Ipv4Addr) -> &mut Self {
        self.addr = addr;
        self
    }

    /// Adds a port to ACM config.
    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    /// Adds a login usermame to ACM config.
    pub fn with_user(&mut self, user: &str) -> &mut Self {
        self.user = user.into();
        self
    }

    /// Adds a login password to ACM config.
    pub fn with_pass(&mut self, pass: &str) -> &mut Self {
        self.pass = pass.into();
        self
    }

    /// Opens a readable and writable SSH stream into an ACM's Site Administration Terminal.
    fn open_stream(&self, term: &[u8]) -> Result<Stream> {
        // Initialize SSH session.
        let stream = TcpStream::connect((self.addr, self.port.unwrap_or(DEFAULT_PORT)))
            .with_context(|| "Failed to open TCP stream to host. Make sure the config is correct and the host is otherwise reachable.")?;
        let mut session = Session::new().with_context(|| "Failed to start SSH session.")?;
        session.set_tcp_stream(stream);
        session.set_timeout(TIMEOUT_MS);
        session
            .handshake()
            .with_context(|| "SSH handshake failed.")?;
        session
            .userauth_keyboard_interactive(&self.user, &mut SshPrompter::new(&self.pass))
            .with_context(|| "Interactive SSH keyboard authentication failed.")?;

        // Open shell on SSH channel.
        let mut channel = session
            .channel_session()
            .with_context(|| "Failed to open SSH channel on SSH session.")?;
        channel
            .request_pty(TERM, None, Some(TERM_DIMS))
            .with_context(|| "Failed to open PTY on SSH channel.")?;
        channel
            .shell()
            .with_context(|| "Failed to open shell on SSH channel.")?;
        channel
            .write_all(term)
            .with_context(|| "Failed to send OSSI term.")?;

        // Waits until OSSI terminator is read to return channel. If this times
        // out, something went wrong with login or ACM is unreachable.
        let mut lines = BufReader::new(channel.stream(0)).lines();
        while let Some(Ok(line)) = lines.next() {
            if line == "t" {
                return Ok(channel.stream(0));
            }
        }

        Err(anyhow!("Never reached OSSI term prompt."))
    }

    /// Runs a given collection of [Message]s (OSSI commands) on the ACM and returns their resulting output.
    pub fn run(&self, inputs: &[Message]) -> Result<Vec<Message>> {
        info!(
            "begin commands for {:?} on thread {:?} of {:?}",
            &self.addr,
            rayon::current_thread_index(),
            rayon::current_num_threads()
        );
        let inputs: String = inputs.iter().map(Message::to_string).collect();
        let mut stream = self.open_stream(OSSI_TERM)?;
        write!(stream, "{}", inputs).with_context(|| "Failed to write inputs to OSSI stream.")?;
        stream
            .write_all(OSSI_LOGOFF)
            .with_context(|| "Failed to write LOGOFF to OSSI stream.")?;
        info!("end command for {:?}", &self.addr);
        Message::from_output(stream)
    }

    /// Like [Self::run], but caches results with a timed cache of thirty minutes.
    pub fn run_cached(&self, inputs: &[Message]) -> Result<Vec<Message>> {
        Ok(run_cached(self, inputs)?.value)
    }

    /// Like [Self::run], but instead of running [Message]s, it returns the manual pages for the provided OSSI commands.
    pub fn manual(&self, inputs: &[Message]) -> Result<String> {
        let inputs: String = inputs.iter().map(Message::to_string).collect();
        let mut stream = self.open_stream(OSSI_MAN_TERM)?;
        write!(stream, "{}", inputs).with_context(|| "Failed to write inputs to OSSI stream.")?;
        stream
            .write_all(OSSI_LOGOFF)
            .with_context(|| "Failed to write LOGOFF to OSSI stream.")?;
        let mut output = String::new();
        stream
            .read_to_string(&mut output)
            .with_context(|| "Failed to read manual pages to string.")?;
        Ok(output)
    }

    /// Reads from `readable`, parsing lines as `asa.cfg`-formatted ACM logins.
    /// Returns a collection of the parsed logins and their associated
    /// names/labels. The spec looks like this:
    ///
    /// ```text
    /// (name user:pass@addr:optional_port)
    /// ACM01 admin:secret@192.168.1.1:5022
    /// ACM02 acdmin:secret@192.168.1.2
    /// ACM03 acdmin:secret@192.168.1.3:5023
    /// ```
    ///
    /// The port is optional. If it is not provided, the default SAT port of
    /// 5022 will be used.
    pub fn from_logins(readable: impl Read) -> Result<Vec<(String, Self)>> {
        let mut acms = Vec::new();

        for line in BufReader::new(readable).lines() {
            let line = line.with_context(|| "Failed to read line of config.")?;
            let mut acm = Self::default();

            if let Some((name, config)) = line.split_once(' ') {
                if let Some((creds, dest)) = config.split_once('@') {
                    if let Some((user, pass)) = creds.split_once(':') {
                        if let Some((addr, port)) = dest.split_once(':') {
                            acm.with_port(
                                port.parse()
                                    .with_context(|| "Failed to parse ACM socket port.")?,
                            )
                            .with_addr(
                                addr.parse()
                                    .with_context(|| "Failed to parse ACM IP address.")?,
                            );
                        } else {
                            acm.with_addr(
                                dest.parse()
                                    .with_context(|| "Failed to parse ACM IP address.")?,
                            );
                        }

                        acm.with_user(user).with_pass(pass);
                        acms.push((name.into(), acm));
                    }
                }
            }
        }
        Ok(acms)
    }
}

/// This memoized function is a timed cache where entries are evicted after
/// thirty minutes. Errors are not cached, only successes.
#[cached(
    result = true,
    type = "TimedCache<String, Return<Vec<Message>>>",
    create = "{ TimedCache::with_lifespan(1800) }",
    convert = r#"{ format!("{}{:?}{:?}", acm.addr, acm.port, inputs) }"#
)]
fn run_cached(acm: &Acm, inputs: &[Message]) -> Result<Return<Vec<Message>>> {
    Ok(Return::new(acm.run(inputs)?))
}

/// Used internally for password-based SSH authentication.
struct SshPrompter<'a> {
    pass: &'a str,
}

impl<'a> SshPrompter<'a> {
    fn new(pass: &'a str) -> Self {
        Self { pass }
    }
}

impl<'a> KeyboardInteractivePrompt for SshPrompter<'a> {
    fn prompt<'b>(
        &mut self,
        _username: &str,
        _instructions: &str,
        _prompts: &[Prompt<'b>],
    ) -> Vec<String> {
        vec![self.pass.to_owned()]
    }
}

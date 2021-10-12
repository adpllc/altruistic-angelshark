use anyhow::{Context, Result};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::{BufRead, BufReader, Read},
};

/// OSSI Messaging Delimiters
const ACM_D: &str = "a";
const COMMAND_D: &str = "c";
const DATA_D: &str = "d";
const ERROR_D: &str = "e";
const FIELD_D: &str = "f";
const NEW_DATA_D: &str = "n";
const TERMINATOR_D: &str = "t";
const TAB: &str = "\t";

/// An OSSI protocol message. Used for input and output to and from the ACM. The
/// OSSI protocol is proprietary, and little documented. Here is a brief
/// overview. Every message consists of a single command and a
/// terminator used to separate messages. The message may also carry optional
/// tab-separated fields and datas when used for input. When used for output, it may
/// optionally carry an error. Example:
///
/// ```text
/// (input)
/// clist object
/// t
/// (output)
/// clist object
/// fhex_code\thex_code\thex_code
/// dentry\tentry\tentry
/// n
/// dentry\tentry\tentry
/// t
/// ```
#[derive(Debug, Default, Clone)]
pub struct Message {
    pub command: String,
    pub fields: Option<Vec<String>>,
    pub datas: Option<Vec<Vec<String>>>,
    pub error: Option<String>,
}

impl Message {
    /// Creates a new [Message] from its basic part: the command.
    pub fn new(command: &str) -> Self {
        Self {
            command: command.into(),
            ..Default::default()
        }
    }

    fn add_fields(&mut self, fields: Vec<String>) -> &mut Self {
        if !fields.is_empty() {
            if let Some(ref mut existing) = self.fields {
                existing.extend(fields.into_iter());
            } else {
                self.fields = Some(fields);
            }
        }

        self
    }

    fn add_data_entry(&mut self, data: Vec<String>) -> &mut Self {
        if !data.is_empty() {
            if let Some(ref mut existing) = self.datas {
                existing.push(data);
            } else {
                self.datas = Some(vec![data]);
            }
        }

        self
    }

    /// Reads from `readable`, parsing lines as Angelshark-formatted OSSI input.
    /// This closely follows the OSSI spec, but also parses ACM names/labels on
    /// lines beginning with `'a'`. The spec looks like this:
    ///
    /// ```text
    /// aACM01
    /// clist object
    /// fhex_code\thex_code\thex_code
    /// dentry\tentry\tentry
    /// t
    /// ...
    /// ```
    pub fn from_input(readable: impl Read) -> Result<Vec<(String, Self)>> {
        let mut data = Vec::new();
        let mut input = Self::default();
        let mut inputs = Vec::new();
        let mut names: Vec<String> = Vec::new();

        for line in BufReader::new(readable).lines() {
            let line = line.with_context(|| "Failed to read line of input.")?;
            let (delim, content) = (line.get(0..1).unwrap_or_default(), line.get(1..));

            match (delim, content) {
                (ACM_D, Some(a)) => {
                    names.extend(a.split(TAB).map(String::from));
                }
                (COMMAND_D, Some(c)) => {
                    input.command = c.into();
                }
                (FIELD_D, Some(f)) => {
                    input.add_fields(f.split(TAB).map(String::from).collect());
                }
                (DATA_D, Some(d)) => {
                    data.extend(d.split(TAB).map(String::from));
                }
                (TERMINATOR_D, _) => {
                    input.add_data_entry(data);
                    inputs.extend(names.into_iter().map(|n| (n, input.clone())));
                    names = Vec::new();
                    input = Message::default();
                    data = Vec::new();
                }
                _ => {
                    // Skip blank lines and unknown identifiers.
                }
            }
        }

        Ok(inputs)
    }

    /// Reads `readable` and parses lines as ACM OSSI output. This should exactly follow the OSSI spec.
    pub fn from_output(readable: impl Read) -> Result<Vec<Self>> {
        let mut data = Vec::new();
        let mut output = Self::default();
        let mut outputs = Vec::new();

        for line in BufReader::new(readable).lines() {
            let line = line.with_context(|| "Failed to read line of output.")?;
            let (delim, content) = (line.get(0..1).unwrap_or_default(), line.get(1..));

            match (delim, content) {
                (COMMAND_D, Some(c)) => {
                    output.command = c.into();
                }
                (ERROR_D, Some(e)) => {
                    output.error = Some(e.into());
                }
                (FIELD_D, Some(f)) => {
                    output.add_fields(f.split(TAB).map(String::from).collect());
                }
                (DATA_D, Some(d)) => {
                    data.extend(d.split(TAB).map(String::from));
                }
                (NEW_DATA_D, _) => {
                    output.add_data_entry(data);
                    data = Vec::new();
                }
                (TERMINATOR_D, _) => {
                    output.add_data_entry(data);
                    data = Vec::new();
                    outputs.push(output);
                    output = Self::default();
                }
                _ => {
                    // Ignore unknown identifiers and blank lines.
                }
            }
        }

        Ok(outputs)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut message = format!("c{}\n", self.command);

        if let Some(error) = &self.error {
            message = message + &format!("e{}\n", error);
        }

        if let Some(fields) = &self.fields {
            message = message + &format!("f{}\n", fields.join(TAB));
        }

        if let Some(datas) = &self.datas {
            message = message
                + &datas
                    .iter()
                    .map(|d| format!("d{}\n", d.join(TAB)))
                    .collect::<Vec<String>>()
                    .join("n\n");
        }

        writeln!(f, "{}\nt", message)
    }
}

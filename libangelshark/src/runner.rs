use crate::{Acm, Message};
use anyhow::Result;
use rayon::iter::IntoParallelIterator;
pub use rayon::iter::ParallelIterator;
use std::collections::HashMap;

/// Allows for more convenient running of OSSI [Message]s on one or more [Acm]s,
/// parallelizing over the ACMs and (optionally) caching results for faster future runs.
///
/// This is the intended high-level use of Angelshark. It holds a collection of
/// "jobs", which are tagged with ACM names/labels and their associated logins ([Acm]s) and [Message]s).
#[derive(Default, Debug, Clone)]
pub struct AcmRunner(HashMap<String, (Acm, Vec<Message>)>);

impl AcmRunner {
    /// Constructs a new [AcmRunner] from tagged [Acm]s and [Message]s.
    pub fn new(acms: Vec<(String, Acm)>, inputs: Vec<(String, Message)>) -> Self {
        let mut runner = AcmRunner::default();
        for (name, acm) in acms {
            runner.register_acm(&name, acm);
        }
        for (name, input) in inputs {
            runner.queue_input(&name, &input);
        }
        runner
    }

    /// Registers an [Acm] as `job_name` in the runner.
    pub fn register_acm(&mut self, job_name: &str, acm: Acm) -> &mut Self {
        self.0.insert(job_name.into(), (acm, Vec::new()));
        self
    }

    /// Queues a [Message] to be run on an [Acm] registered as `job_name`.
    pub fn queue_input(&mut self, job_name: &str, input: &Message) -> &mut Self {
        if let Some((_, inputs)) = self.0.get_mut(job_name) {
            inputs.push(input.clone());
        }
        self
    }

    /// Runs the queued [Message] inputs on the registered [Acm]s and returns
    /// the results. The results are returned as an iterator. The iterator must
    /// be in some way consumed, collected, or iterated over before the runner
    /// starts running commands, i.e. it is lazy. Once this begins, results are
    /// computed in parallel over the ACMs. The order of outputs is undefined.
    pub fn run(self) -> impl ParallelIterator<Item = RunOutput> {
        self.0
            .into_par_iter()
            .filter(|(_, (_, inputs))| !inputs.is_empty())
            .map(|(job_name, (acm, inputs))| (job_name, acm.run(&inputs)))
    }

    /// Functionally equivalent to [Self::run] but caches results for 30 minutes
    /// to make future lookups faster.
    pub fn run_cached(self) -> impl ParallelIterator<Item = RunOutput> {
        self.0
            .into_par_iter()
            .filter(|(_, (_, inputs))| !inputs.is_empty())
            .map(|(job_name, (acm, inputs))| (job_name, acm.run_cached(&inputs)))
    }

    /// Functionally equivalent to [Self::run] but returns manual pages for
    /// inputs instead of executing them.
    pub fn manuals(self) -> impl ParallelIterator<Item = ManualOutput> {
        self.0
            .into_par_iter()
            .filter(|(_, (_, inputs))| !inputs.is_empty())
            .map(|(job_name, (acm, inputs))| (job_name, acm.manual(&inputs)))
    }
}

/// Every resulting entry of [AcmRunner::run]
pub type RunOutput = (String, Result<Vec<Message>>);

/// Every resulting entry of [AcmRunner::manuals]
pub type ManualOutput = (String, Result<String>);

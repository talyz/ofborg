use tasks::eval::{EvaluationStrategy, StepResult};
use std::path::Path;
use ofborg::commitstatus::CommitStatus;

pub struct GenericStrategy {}
impl GenericStrategy {
    pub fn new() -> GenericStrategy {
        Self {}
    }
}

impl EvaluationStrategy for GenericStrategy {
    fn pre_clone(&self) -> StepResult {
        Ok(())
    }

    fn before_merge(&self, co: &Path,  status: &mut CommitStatus) -> StepResult {
        Ok(())
    }
}

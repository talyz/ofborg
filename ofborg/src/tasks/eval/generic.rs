use tasks::eval::{EvaluationStrategy, StepResult};
use std::path::Path;
use ofborg::commitstatus::CommitStatus;
use ofborg::checkout::CachedProjectCo;

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

    fn on_target_branch(&self, co: &Path,  status: &mut CommitStatus) -> StepResult {
        Ok(())
    }

    fn after_fetch(&self, co: &CachedProjectCo) -> StepResult {
        Ok(())
    }
}

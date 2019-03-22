use tasks::eval::{EvaluationStrategy, StepResult};
use std::path::Path;
use ofborg::commitstatus::CommitStatus;
use ofborg::checkout::CachedProjectCo;
use ofborg::evalchecker::EvalChecker;
use ofborg::message::buildjob::BuildJob;

pub struct GenericStrategy {}
impl GenericStrategy {
    pub fn new() -> GenericStrategy {
        Self {}
    }
}

impl EvaluationStrategy for GenericStrategy {
    fn pre_clone(&self) -> StepResult<()> {
        Ok(())
    }

    fn on_target_branch(&self, co: &Path,  status: &mut CommitStatus) -> StepResult<()> {
        Ok(())
    }

    fn after_fetch(&self, co: &CachedProjectCo) -> StepResult<()> {
        Ok(())
    }

    fn merge_conflict(&self) {
    }

    fn after_merge(&self, status: &mut CommitStatus) -> StepResult<()> {
        Ok(())
    }

    fn evaluation_checks(&self) -> Vec<EvalChecker> {
        vec![]
    }

    fn all_evaluations_passed(&self) -> StepResult<Vec<BuildJob>> {
        Ok(vec![])
    }
}

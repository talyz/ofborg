pub mod stdenvs;
pub use self::stdenvs::Stdenvs;
mod nixpkgs;
pub use self::nixpkgs::NixpkgsStrategy;
mod generic;
pub use self::generic::GenericStrategy;
use std::path::Path;
use ofborg::commitstatus::CommitStatus;
use ofborg::checkout::CachedProjectCo;
use ofborg::evalchecker::EvalChecker;
use ofborg::message::buildjob::BuildJob;

pub trait EvaluationStrategy {
    fn pre_clone(&self) -> StepResult<()>;
    fn on_target_branch(&self, co: &Path, status: &mut CommitStatus) -> StepResult<()>;
    fn after_fetch(&self, co: &CachedProjectCo) -> StepResult<()>;
    fn merge_conflict(&self);
    fn after_merge(&self, status: &mut CommitStatus) -> StepResult<()>;
    fn evaluation_checks(&self) -> Vec<EvalChecker>;
    fn all_evaluations_passed(&self) -> StepResult<Vec<BuildJob>>;
}

type StepResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Fail(String),
    FailWithGist(String, String, String),
}

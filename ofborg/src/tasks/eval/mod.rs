pub mod stdenvs;
pub use self::stdenvs::Stdenvs;
mod nixpkgs;
pub use self::nixpkgs::NixpkgsStrategy;
mod generic;
pub use self::generic::GenericStrategy;
use std::path::Path;
use ofborg::commitstatus::CommitStatus;
use ofborg::checkout::CachedProjectCo;

pub trait EvaluationStrategy {
    fn pre_clone(&self) -> StepResult;
    fn on_target_branch(&self, co: &Path, status: &mut CommitStatus) -> StepResult;
    fn after_fetch(&self, co: &CachedProjectCo) -> StepResult;
}

type StepResult = Result<(), Error>;
#[derive(Debug)]
pub enum Error {
    Fail(String),
}

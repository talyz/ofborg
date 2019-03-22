pub mod stdenvs;
pub use self::stdenvs::Stdenvs;
mod nixpkgs;
pub use self::nixpkgs::NixpkgsStrategy;
mod generic;
pub use self::generic::GenericStrategy;
use std::path::Path;
use ofborg::commitstatus::CommitStatus;

pub trait EvaluationStrategy {
    fn pre_clone(&self) -> StepResult;
    fn before_merge(&self, co: &Path, status: &mut CommitStatus) -> StepResult;
}

type StepResult = Result<(), Error>;
#[derive(Debug)]
pub enum Error {
    Fail(String),
}

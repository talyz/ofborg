pub mod stdenvs;
pub use self::stdenvs::Stdenvs;
mod nixpkgs;
pub use self::nixpkgs::NixpkgsStrategy;
mod generic;
pub use self::generic::GenericStrategy;

pub trait EvaluationStrategy {
    fn pre_clone(&self) -> StepResult;
    fn before_merge(&self, status: ()) -> StepResult;
}

type StepResult = Result<(), Error>;
pub enum Error {
    Fail(String),
}

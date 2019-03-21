pub mod stdenvs;
pub use self::stdenvs::Stdenvs;
mod nixpkgs;
pub use self::nixpkgs::NixpkgsStrategy;
mod generic;
pub use self::generic::GenericStrategy;

pub trait EvaluationStrategy {
    fn pre_clone(&self) -> Result<(), ()>;
}

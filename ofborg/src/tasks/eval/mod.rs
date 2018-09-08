pub struct PRStatus {
    //context: String,
    //description: String,
}

pub trait StraddledEvaluationTask: Sized {
    fn before_on_target_branch_message(&self) -> String;
    fn on_target_branch(&mut self);

    fn before_after_merge_message(&self) -> String;
    fn after_merge(&mut self);
}

pub mod maintainers;

pub mod stdenvs;
pub use self::stdenvs::Stdenvs;

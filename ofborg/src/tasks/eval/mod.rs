pub struct EvaluationResult {
    pub tags: Option<TagDiff>,
}

pub struct TagDiff {
    pub add: Vec<String>,
    pub delete: Vec<String>,
}

pub trait StraddledEvaluationTask: Sized {
    fn before_on_target_branch_message(&self) -> String;
    fn on_target_branch(&mut self);

    fn before_after_merge_message(&self) -> String;
    fn after_merge(&mut self);

    fn results(self) -> EvaluationResult;
}

pub mod maintainers;

pub mod stdenvs;
pub use self::stdenvs::Stdenvs;

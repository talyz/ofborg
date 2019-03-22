use tasks::eval::{EvaluationStrategy, StepResult};

pub struct GenericStrategy {}
impl GenericStrategy {
    pub fn new() -> GenericStrategy {
        Self {}
    }
}

impl EvaluationStrategy for GenericStrategy {
    fn pre_clone(&self) -> StepResult {
    }

    fn before_merge(&self, status: ()) -> StepResult {

    }
}

use tasks::eval::EvaluationStrategy;

pub struct GenericStrategy {}
impl GenericStrategy {
    pub fn new() -> GenericStrategy {
        Self {}
    }
}

impl EvaluationStrategy for GenericStrategy {
    fn pre_clone(&self) -> Result<(), ()> {
        Ok(())
    }
}

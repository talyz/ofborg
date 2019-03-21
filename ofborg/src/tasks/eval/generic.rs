use tasks::eval::EvaluationStrategy;

pub struct GenericStrategy {}
impl GenericStrategy {
    pub fn new() -> Box<EvaluationStrategy> {
        Box::new(Self {})
    }
}

impl EvaluationStrategy for GenericStrategy {}

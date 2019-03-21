use tasks::eval::EvaluationStrategy;

pub struct NixpkgsStrategy {}

impl NixpkgsStrategy {
    pub fn new() -> Box<EvaluationStrategy> {
        Box::new(Self {})
    }
}

impl EvaluationStrategy for NixpkgsStrategy {}

use tasks::eval::{EvaluationTask, PRStatus};

pub struct StdenvHashes {

}

impl StdenvHashes {
    pub fn new() -> StdenvHashes {
        StdenvHashes {

        }
    }
}

impl EvaluationTask for StdenvHashes {
    fn before_merge(&mut self) -> Option<PRStatus> {
        None
    }
    fn after_merge(&mut self) -> Option<PRStatus> {
        None
    }
}

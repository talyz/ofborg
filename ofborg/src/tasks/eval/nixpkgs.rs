use tasks::eval::{EvaluationStrategy, StepResult, Stdenvs};
use hubcaps::issues::IssueRef;
use tasks::evaluate::update_labels;

pub struct NixpkgsStrategy<'a> {
    issue: &'a IssueRef<'a>,
}

impl <'a> NixpkgsStrategy<'a> {
    pub fn new(issue: &'a IssueRef) -> NixpkgsStrategy<'a> {
        Self {
            issue
        }
    }

    fn tag_from_title(&self) {
        let darwin = self.issue.get()
            .map(|iss| {
                iss.title.to_lowercase().contains("darwin")
                    || iss.title.to_lowercase().contains("macos")
            })
            .unwrap_or(false);

        if darwin {
            update_labels(&self.issue, &[String::from("6.topic: darwin")], &[]);
        }
    }
}

impl <'a> EvaluationStrategy for NixpkgsStrategy<'a> {
    fn pre_clone(&self) -> StepResult {
        self.tag_from_title();
        Ok(())
    }
}

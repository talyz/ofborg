use tasks::eval::{EvaluationStrategy, StepResult, Stdenvs, Error};
use hubcaps::issues::IssueRef;
use hubcaps::gists::Gists;
use tasks::evaluate::{update_labels, make_gist};
use std::path::PathBuf;
use ofborg::outpathdiff::{OutPathDiff, OutPaths};
use std::time::Instant;
use ofborg::stats::Event;
use std::path::Path;
use ofborg::files::file_to_str;
use ofborg::message::evaluationjob::EvaluationJob;
use ofborg::nix::Nix;
use ofborg::commitstatus::CommitStatus;

pub struct NixpkgsStrategy<'a> {
    job: &'a EvaluationJob,
    issue: &'a IssueRef<'a>,
    gists: &'a Gists<'a>,
    events: (),
    nix: &'a Nix,
}

impl <'a> NixpkgsStrategy<'a> {
    pub fn new(job: &'a EvaluationJob, issue: &'a IssueRef, gists: &'a Gists, nix: &'a Nix) -> NixpkgsStrategy<'a> {
        Self {
            job,
            issue,
            gists,
            nix,
            events: ()
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

    fn on_target_branch(&self, co: &Path, status: &mut CommitStatus) -> StepResult {
        status.set_with_description(
            "Checking original stdenvs",
            hubcaps::statuses::State::Pending,
        );

        let mut stdenvs = Stdenvs::new(self.nix.clone(), PathBuf::from(&co));
        stdenvs.identify_before();

        let mut rebuildsniff = OutPathDiff::new(self.nix.clone(), PathBuf::from(&co));

        status.set_with_description(
            "Checking original out paths",
            hubcaps::statuses::State::Pending,
        );


        let target_branch = match self.job.pr.target_branch.clone() {
            Some(x) => x,
            None => String::from("master"),
        };

        let target_branch_rebuild_sniff_start = Instant::now();

        if let Err(mut output) = rebuildsniff.find_before() {
            status.set_url(make_gist(
                &self.gists,
                "Output path comparison",
                Some("".to_owned()),
                file_to_str(&mut output),
            ));

            /*
            self.events
                .notify(Event::TargetBranchFailsEvaluation(target_branch.clone()));
*/
            status.set_with_description(
                format!("Target branch {} doesn't evaluate!", &target_branch).as_ref(),
                hubcaps::statuses::State::Failure,
            );

            return Err(Error::Fail(String::from("Pull request targets a branch which does not evaluate!")))
        }

/*
        self.events.notify(Event::EvaluationDuration(
            target_branch.clone(),
            target_branch_rebuild_sniff_start.elapsed().as_secs(),
        ));
        self.events
            .notify(Event::EvaluationDurationCount(target_branch.clone()));
*/
        Ok(())
    }
}

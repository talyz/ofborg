use ofborg::tagger::{
    MaintainerPRTagger, PathsTagger, PkgsAddedRemovedTagger, RebuildTagger, StdenvTagger,
};
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
use ofborg::checkout::CachedProjectCo;
use std::collections::HashMap;

pub struct NixpkgsStrategy<'a> {
    job: &'a EvaluationJob,
    issue: &'a IssueRef<'a>,
    gists: &'a Gists<'a>,
    events: (),
    nix: &'a Nix,
    tag_paths: &'a HashMap<String, Vec<String>>,
}

impl <'a> NixpkgsStrategy<'a> {
    pub fn new(job: &'a EvaluationJob, issue: &'a IssueRef, gists: &'a Gists, nix: &'a Nix, tag_paths: &'a HashMap<String, Vec<String>>,) -> NixpkgsStrategy<'a> {
        Self {
            job,
            issue,
            gists,
            nix,
            tag_paths,
            events: (),
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

    fn tag_from_paths(&self, issue: &hubcaps::issues::IssueRef, paths: &[String]) {
        let mut tagger = PathsTagger::new(self.tag_paths.clone());

        for path in paths {
            tagger.path_changed(&path);
        }

        update_labels(&issue, &tagger.tags_to_add(), &tagger.tags_to_remove());
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

    fn after_fetch(&self, co: &CachedProjectCo) -> StepResult {
        let possibly_touched_packages = parse_commit_messages(
            &co.commit_messages_from_head(&self.job.pr.head_sha)
                .unwrap_or_else(|_| vec!["".to_owned()]),
        );

        let changed_paths = co
            .files_changed_from_head(&self.job.pr.head_sha)
            .unwrap_or_else(|_| vec![]);
        self.tag_from_paths(&self.issue, &changed_paths);

        Ok(())
    }



    fn merge_conflict(&self) {
        update_labels(&self.issue, &["2.status: merge conflict".to_owned()], &[]);
    }

    fn after_merge(&self) -> StepResult {
        update_labels(&self.issue, &[], &["2.status: merge conflict".to_owned()]);

        Ok(())
    }
}


fn parse_commit_messages(messages: &[String]) -> Vec<String> {
    messages
        .iter()
        .filter_map(|line| {
            // Convert "foo: some notes" in to "foo"
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some(parts[0])
            } else {
                None
            }
        })
        .flat_map(|line| {
            let pkgs: Vec<&str> = line.split(',').collect();
            pkgs
        })
        .map(|line| line.trim().to_owned())
        .collect()
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_commit_messages() {
        let expect: Vec<&str> = vec![
            "firefox{-esr", // don't support such fancy syntax
            "}",            // Don't support such fancy syntax
            "firefox",
            "buildkite-agent",
            "python.pkgs.ptyprocess",
            "python.pkgs.ptyprocess",
            "android-studio-preview",
            "foo",
            "bar",
        ];
        assert_eq!(
            parse_commit_messages(
                &"
              firefox{-esr,}: fix failing build due to the google-api-key
              Merge pull request #34483 from andir/dovecot-cve-2017-15132
              firefox: enable official branding
              Merge pull request #34442 from rnhmjoj/virtual
              buildkite-agent: enable building on darwin
              python.pkgs.ptyprocess: 0.5 -> 0.5.2
              python.pkgs.ptyprocess: move expression
              Merge pull request #34465 from steveeJ/steveej-attempt-qtile-bump-0.10.7
              android-studio-preview: 3.1.0.8 -> 3.1.0.9
              Merge pull request #34188 from dotlambda/home-assistant
              Merge pull request #34414 from dotlambda/postfix
              foo,bar: something here: yeah
            "
                .lines()
                .map(|l| l.to_owned())
                .collect::<Vec<String>>(),
            ),
            expect
        );
    }
}

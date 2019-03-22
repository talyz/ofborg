use ofborg::tagger::{
    MaintainerPRTagger, PathsTagger, PkgsAddedRemovedTagger, RebuildTagger, StdenvTagger,
};
use ofborg::worker;
use ofborg::message::{buildjob, evaluationjob};
use ofborg::commentparser::Subset;
use uuid::Uuid;
use tasks::eval::{EvaluationStrategy, StepResult, Stdenvs, Error};
use hubcaps::issues::IssueRef;
use hubcaps::gists::Gists;
use hubcaps::repos::Repo;
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
use ofborg::evalchecker::EvalChecker;
use ofborg::nix;
use ofborg::message::buildjob::BuildJob;

pub struct NixpkgsStrategy<'a> {
    job: &'a EvaluationJob,
    repo: &'a Repo<'a>,
    issue: &'a IssueRef<'a>,
    gists: &'a Gists<'a>,
    events: (),
    nix: &'a Nix,
    tag_paths: &'a HashMap<String, Vec<String>>,
    stdenvs: Option<Stdenvs>,
    outpathdiff: Option<OutPathDiff>,
    resulting_actions: worker::Actions,
    possibly_touched_packages: Option<Vec<String>>,
}

impl <'a> NixpkgsStrategy<'a> {
    pub fn new(job: &'a EvaluationJob, repo: &'a Repo, issue: &'a IssueRef, gists: &'a Gists, nix: &'a Nix, tag_paths: &'a HashMap<String, Vec<String>>,) -> NixpkgsStrategy<'a> {
        Self {
            job,
            repo,
            issue,
            gists,
            nix,
            tag_paths,
            events: (),
            stdenvs: None,
            outpathdiff: None,
            resulting_actions: vec![],
            possibly_touched_packages: None,
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

    fn check_meta(&self) -> StepResult<Vec<BuildJob>> {
        let mut status = CommitStatus::new(
            self.repo.statuses(),
            self.job.pr.head_sha.clone(),
            String::from("grahamcofborg-eval-check-meta"),
            String::from("config.nix: checkMeta = true"),
            None,
        );

        status.set(hubcaps::statuses::State::Pending);

        let state: hubcaps::statuses::State;
        let gist_url: Option<String>;

        let checker = OutPaths::new(self.nix.clone(), PathBuf::from(&refpath), true);
        match checker.find() {
            Ok(pkgs) => {
                status.set_url(None);
                status.set(hubcaps::statuses::State::Success);

                if let Some(possibly_touched_packages) = self.possibly_touched_packages {
                    let mut try_build: Vec<String> = pkgs
                        .keys()
                        .map(|pkgarch| pkgarch.package.clone())
                        .filter(|pkg| possibly_touched_packages.contains(&pkg))
                        .collect();
                    try_build.sort();
                    try_build.dedup();

                    if !try_build.is_empty() && try_build.len() <= 10 {
                        // In the case of trying to merge master in to
                        // a stable branch, we don't want to do this.
                        // Therefore, only schedule builds if there
                        // less than or exactly 10
                        let msg = buildjob::BuildJob::new(
                            self.job.repo.clone(),
                            self.job.pr.clone(),
                            Subset::Nixpkgs,
                            try_build,
                            None,
                            None,
                            format!("{}", Uuid::new_v4()),
                        );

                        return Ok(vec![ msg ]);
                    }
                }

                return Ok(vec![]);
            }
            Err(mut out) => {
                state = hubcaps::statuses::State::Failure;
                gist_url = make_gist(
                    &self.gists,
                    "Meta Check",
                    Some(format!("{:?}", state)),
                    file_to_str(&mut out),
                );

                status.set_url(gist_url);
                status.set(state);
                return Err(Error::Fail(String::from("Failed to run verify package meta fields.")));
            }
        }
    }
}

impl <'a> EvaluationStrategy for NixpkgsStrategy<'a> {
    fn pre_clone(&self) -> StepResult<()> {
        self.tag_from_title();
        Ok(())
    }

    fn on_target_branch(&self, co: &Path, status: &mut CommitStatus) -> StepResult<()> {
        status.set_with_description(
            "Checking original stdenvs",
            hubcaps::statuses::State::Pending,
        );

        let mut stdenvs = Stdenvs::new(self.nix.clone(), PathBuf::from(&co));
        stdenvs.identify_before();
        self.stdenvs = Some(stdenvs);

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
        self.outpathdiff = Some(rebuildsniff);

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

    fn after_fetch(&self, co: &CachedProjectCo) -> StepResult<()> {
        self.possibly_touched_packages = Some(parse_commit_messages(
            &co.commit_messages_from_head(&self.job.pr.head_sha)
                .unwrap_or_else(|_| vec!["".to_owned()]),
        ));

        let changed_paths = co
            .files_changed_from_head(&self.job.pr.head_sha)
            .unwrap_or_else(|_| vec![]);
        self.tag_from_paths(&self.issue, &changed_paths);

        Ok(())
    }

    fn merge_conflict(&self) {
        update_labels(&self.issue, &["2.status: merge conflict".to_owned()], &[]);
    }

    fn after_merge(&self, status: &mut CommitStatus) -> StepResult<()> {
        update_labels(&self.issue, &[], &["2.status: merge conflict".to_owned()]);


        if let Some(stdenvs) = self.stdenvs {
            status
                .set_with_description("Checking new stdenvs", hubcaps::statuses::State::Pending);
            stdenvs.identify_after();
        }

        if let Some(rebuildsniff) = self.outpathdiff {
            status
                .set_with_description("Checking new out paths", hubcaps::statuses::State::Pending);

            if let Err(mut output) = rebuildsniff.find_after() {
                return Err(Error::FailWithGist(
                    String::from("Failed to enumerate outputs after merging to {}"),
                    String::from("Output path comparison"),
                    file_to_str(&mut output)
                ))
            }
        }

        Ok(())
    }

    fn evaluation_checks(&self) -> Vec<EvalChecker> {
        vec![
            EvalChecker::new(
                "package-list",
                nix::Operation::QueryPackagesJSON,
                vec![String::from("--file"), String::from(".")],
                self.nix.clone(),
            ),
            EvalChecker::new(
                "package-list-no-aliases",
                nix::Operation::QueryPackagesJSON,
                vec![
                    String::from("--file"),
                    String::from("."),
                    String::from("--arg"),
                    String::from("config"),
                    String::from("{ allowAliases = false; }"),
                ],
                self.nix.clone(),
            ),
            EvalChecker::new(
                "nixos-options",
                nix::Operation::Instantiate,
                vec![
                    String::from("--arg"),
                    String::from("nixpkgs"),
                    String::from("{ outPath=./.; revCount=999999; shortRev=\"ofborg\"; }"),
                    String::from("./nixos/release.nix"),
                    String::from("-A"),
                    String::from("options"),
                ],
                self.nix.clone(),
            ),
            EvalChecker::new(
                "nixos-manual",
                nix::Operation::Instantiate,
                vec![
                    String::from("--arg"),
                    String::from("nixpkgs"),
                    String::from("{ outPath=./.; revCount=999999; shortRev=\"ofborg\"; }"),
                    String::from("./nixos/release.nix"),
                    String::from("-A"),
                    String::from("manual"),
                ],
                self.nix.clone(),
            ),
            EvalChecker::new(
                "nixpkgs-manual",
                nix::Operation::Instantiate,
                vec![
                    String::from("--arg"),
                    String::from("nixpkgs"),
                    String::from("{ outPath=./.; revCount=999999; shortRev=\"ofborg\"; }"),
                    String::from("./pkgs/top-level/release.nix"),
                    String::from("-A"),
                    String::from("manual"),
                ],
                self.nix.clone(),
            ),
            EvalChecker::new(
                "nixpkgs-tarball",
                nix::Operation::Instantiate,
                vec![
                    String::from("--arg"),
                    String::from("nixpkgs"),
                    String::from("{ outPath=./.; revCount=999999; shortRev=\"ofborg\"; }"),
                    String::from("./pkgs/top-level/release.nix"),
                    String::from("-A"),
                    String::from("tarball"),
                ],
                self.nix.clone(),
            ),
            EvalChecker::new(
                "nixpkgs-unstable-jobset",
                nix::Operation::Instantiate,
                vec![
                    String::from("--arg"),
                    String::from("nixpkgs"),
                    String::from("{ outPath=./.; revCount=999999; shortRev=\"ofborg\"; }"),
                    String::from("./pkgs/top-level/release.nix"),
                    String::from("-A"),
                    String::from("unstable"),
                ],
                self.nix.clone(),
            ),
        ]
    }

    fn all_evaluations_passed(&self) -> StepResult<Vec<BuildJob>> {
        self.check_meta()
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

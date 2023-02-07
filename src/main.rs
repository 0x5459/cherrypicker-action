use actions::get_input_required;
use anyhow::{Context, Result};
use github::{client_for, download_patch, ensure_fork};
use octocrab::models::events::payload::{
    EventPayload, IssueCommentEventAction, IssueCommentEventPayload, PullRequestEventPayload,
};
use tokio::task::spawn_blocking;

mod config;
mod github;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    let repo_token: String = get_input_required("repo-token")?;
    octocrab::initialise(octocrab::Octocrab::builder().personal_token(repo_token))
        .context("initialise octocrab")?;

    let ctx = spawn_blocking(actions::Context::from_env)
        .await
        .context("create actions context")?;

    match ctx.payload {
        EventPayload::IssueCommentEvent(evt) => on_issue_comment(evt).await,
        EventPayload::PullRequestEvent(evt) => on_pull_request(evt).await,
        _ => {}
    }
    Ok(())
}

async fn on_issue_comment(event: Box<IssueCommentEventPayload>) {
    // Only consider new comments in PRs.
    if event.action != IssueCommentEventAction::Created || event.issue.pull_request.is_none() {}
}

async fn on_pull_request(_issue_comment_event: Box<PullRequestEventPayload>) {}

async fn cherry_pick(
    forking_user: impl Into<String>,
    owner: impl Into<String>,
    repo: impl Into<String>,
    pull_number: u64,
    target_branch: impl Into<String>,
) -> anyhow::Result<()> {
    let forking_user = forking_user.into();
    let owner = owner.into();
    let repo = repo.into();
    let target_branch = target_branch.into();

    // 1. fork
    ensure_fork(&forking_user, &owner, &repo)
        .await
        .context("ensure fork")?;

    // 2. clone
    let mut git = client_for(&forking_user, &repo).await;

    // 3. checkout to target branch
    git.checkout(&target_branch)
        .await
        .context("checkout to target branch")?;

    // 4. checkout -b
    let local_branch = format!("cherry-pick-{}-to-{}", pull_number, target_branch);
    git.checkout_new_branch(&local_branch)
        .await
        .context("checkout new branch")?;

    // 5. download patch from github
    let patch_path = download_patch(owner, repo, pull_number, &target_branch)
        .await
        .context("download patch")?;

    // 6. git config

    // Title for GitHub issue/PR.
    let title = format!("cherry-pick #{} to {}", pull_number, target_branch);

    // 7. Try git am --3way localPath.
    git.am(patch_path).await.with_context(|| {
        format!(
            "apply #{} on top of target branch {}",
            pull_number, target_branch
        )
    })?;

    // 8. push
    git.push("origin".to_string(), local_branch, true)
        .await
        .context("push to github")?;
    
    // 9. create pr

    todo!()
}

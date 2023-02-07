use std::{env::temp_dir, path::PathBuf, time::Duration};

use anyhow::{anyhow, Context};
use git::{Git, GitBuilder};
use logit::Logit;
use octocrab::models::pulls::PullRequest;
use random_color::RandomColor;
use tokio::{fs, time};

use crate::config::config;

use self::{api::list_repos_for_user, page::list_all};

mod api;
mod page;

pub async fn client_for(owner: impl Into<String>, repo: impl Into<String>) -> Git {
    let owner = owner.into();
    let repo = repo.into();

    let mut git = GitBuilder::new(&owner, &repo).build();
    git.clone(format!("https://github.com/{}/{}.git", owner, repo))
        .await
        .logit("clone repo");
    git
}

// `ensureFork` checks to see that there is a fork of org/repo in the forkedUsers repositories.
// If there is not, it makes one, and waits for the fork to be created before returning.
// The return value is the name of the repo that was created
// (This may be different then the one that is forked due to naming conflict)
pub async fn ensure_fork(
    forking_user: impl Into<String>,
    owner: impl Into<String>,
    repo: impl Into<String>,
) -> anyhow::Result<String> {
    let forking_user = forking_user.into();
    let owner = owner.into();
    let repo = repo.into();
    if is_forked(&forking_user, &owner, &repo).await {
        return Ok(repo);
    }

    let forked = octocrab::instance()
        .repos(&owner, &repo)
        .create_fork()
        .send()
        .await
        .context("create fork")?;

    wait_for_repo(owner, repo).await.context("wait for repo")?;

    Ok(forked.name)
}

// Returns true if forkingUser forked owner/repo
pub async fn is_forked(
    forking_user: impl Into<String>,
    owner: impl Into<String>,
    repo: impl Into<String>,
) -> bool {
    let forking_user = forking_user.into();
    let owner = owner.into();
    let repo = repo.into();

    let fork = format!("{}/{}", forking_user, repo);

    let repos = list_all(list_repos_for_user(&forking_user)).await.unwrap();
    let forked_repo = repos
        .into_iter()
        .find(|repo| repo.fork == Some(true) && repo.full_name.as_ref() == Some(&fork));

    let forked_repo = match forked_repo {
        Some(x) => x,
        None => return false,
    };

    let parent_full_name = octocrab::instance()
        .repos(forking_user, forked_repo.name)
        .get()
        .await
        .map(|r| r.parent.and_then(|p| p.full_name))
        .unwrap();

    match parent_full_name {
        Some(n) => n == format!("{}/{}", owner, repo),
        None => false,
    }
}

pub async fn wait_for_repo(
    owner: impl Into<String>,
    repo: impl Into<String>,
) -> anyhow::Result<()> {
    // Wait for at most 5 minutes for the fork to appear on GitHub.
    // The documentation instructs us to contact support if this
    // takes longer than five minutes.

    let owner = owner.into();
    let repo = repo.into();

    let repo_full_name = format!("{}/{}", owner, repo);

    let mut interval = time::interval(Duration::from_secs(15));
    if let Err(_) = time::timeout(Duration::from_secs(6 * 60), async move {
        loop {
            interval.tick().await;
            match octocrab::instance()
                .repos(owner.clone(), repo.clone())
                .get()
                .await
                .logit_warn("Error getting bot repository.")
            {
                Ok(r) if r.fork == Some(true) => {
                    return;
                }
                _ => continue,
            }
        }
    })
    .await
    {
        return Err(anyhow!(
            "timed out waiting for {} to appear on GitHub",
            repo_full_name
        ));
    }
    Ok(())
}

pub fn is_picked(pr: &PullRequest, target_branch: impl Into<String>) -> bool {
    let labels = match &pr.labels {
        Some(x) => x,
        None => return false,
    };

    let target_branch = target_branch.into();

    let picked_label_prefix = &config().picked_label_prefix;
    labels
        .iter()
        .any(|label| label.name.strip_prefix(picked_label_prefix) == Some(&target_branch))
}

pub async fn ensure_label(
    owner: impl Into<String>,
    repo: impl Into<String>,
    label: impl Into<String>,
) -> anyhow::Result<()> {
    let owner = owner.into();
    let repo = repo.into();
    let label = label.into();

    octocrab::instance()
        .issues(owner, repo)
        .create_label(&label, &RandomColor::new().to_hex()[1..], &label)
        .await
        .context("create label")?;
    Ok(())
}

pub async fn download_patch(
    owner: impl Into<String>,
    repo: impl Into<String>,
    pull_number: u64,
    target_branch: impl Into<String>,
) -> anyhow::Result<PathBuf> {
    let owner = owner.into();
    let repo = repo.into();

    let filename = format!(
        "{}-{}-{}-{}",
        owner,
        repo,
        pull_number,
        normalize(target_branch.into())
    );

    let pr = octocrab::instance()
        .pulls(owner, repo)
        .get(pull_number)
        .await
        .context("get pull request")?;

    let resp = reqwest::get(
        pr.patch_url
            .with_context(|| format!("pull request patch_url is None. {}", filename))?,
    )
    .await
    .unwrap();

    let p = temp_dir().join(&filename);
    fs::write(
        &p,
        resp.bytes()
            .await
            .with_context(|| format!("get the fetch response body {}", filename))?,
    )
    .await
    .with_context(|| format!("write patch file {}", filename))?;

    Ok(p)
}

pub fn normalize(input: impl AsRef<str>) -> String {
    input.as_ref().replace('/', "-")
}

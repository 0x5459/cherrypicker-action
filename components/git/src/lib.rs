use std::{
    env,
    ffi::OsStr,
    future::{ready, Future, Ready},
    io,
    path::{Path, PathBuf},
    process::Output,
};

use anyhow::{Context, Result};
use executor::GitCommandExecutor;
use futures::future::BoxFuture;
use logit::Logit;
use tokio::fs;

pub mod executor;

pub struct GitBuilder {
    owner: String,
    repo: String,
    censor: Option<fn(&OsStr) -> &OsStr>,
    executor: Option<Box<dyn DynExecutor + Send>>,
    dir: Option<PathBuf>,
    git_user_getter: Option<Box<dyn DynGitUserGetter + Send>>,
}

impl GitBuilder {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            censor: None,
            dir: None,
            executor: None,
            git_user_getter: None,
        }
    }

    pub fn censor(mut self, censor: fn(&OsStr) -> &OsStr) -> Self {
        self.censor = Some(censor);
        self
    }

    pub fn executor(mut self, ex: Box<dyn DynExecutor + Send>) -> Self {
        self.executor = Some(ex);
        self
    }

    pub fn dir(mut self, p: PathBuf) -> Self {
        self.dir = Some(p);
        self
    }

    pub fn git_user_getter(mut self, gg: Box<dyn DynGitUserGetter + Send>) -> Self {
        self.git_user_getter = Some(gg);
        self
    }

    pub fn build(self) -> Git {
        let censor = self.censor.unwrap_or(|x: &OsStr| x);
        let executor = self
            .executor
            .unwrap_or_else(|| Box::new(GitCommandExecutor::new()));
        let git_user_getter = self
            .git_user_getter
            .unwrap_or_else(|| Box::new(DefaultGitUserGetter {}));
        let dir = self.dir.unwrap_or_else(|| {
            env::current_dir()
                .unwrap_or(env::temp_dir())
                .join(self.owner)
                .join(self.repo)
        });

        let _ = std::fs::create_dir_all(&dir).logit("create git dir");
        Git {
            dir,
            info: git_user_getter,
            executor: Box::new(CensoringExecutor::new(censor, executor)),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExecutorError {
    #[error("spawn command error {0:?}")]
    SpawnError(io::Error),
    /// Spawn a command succeed, but the command exit status is error.
    #[error("Spawn a command succeed, but the command exit status is error {0:?}")]
    OutputError(Output),
}

pub trait Executor {
    type Fut<'a>: Future<Output = Result<Output, ExecutorError>> + Send + 'a
    where
        Self: 'a;

    fn exec(&self, args: &[&OsStr]) -> Self::Fut<'_>;
}

pub trait DynExecutor {
    fn exec(&self, args: &[&OsStr]) -> BoxFuture<Result<Output, ExecutorError>>;
}

impl<EX: Executor> DynExecutor for EX {
    fn exec(&self, args: &[&OsStr]) -> BoxFuture<Result<Output, ExecutorError>> {
        Box::pin(Executor::exec(self, args))
    }
}

// impl Executor for Box<dyn DynExecutor> {
//     type Fut = BoxFuture<'static, Result<Output, ExecutorError>>;

//     fn exec(&self, args: &[&OsStr]) -> Self::Fut {
//         DynExecutor::exec(self, args)
//     }
// }

struct CensoringExecutor {
    // censor censors content to remove secrets
    censor: fn(&OsStr) -> &OsStr,
    inner: Box<dyn DynExecutor + Send>,
}

impl CensoringExecutor {
    pub fn new(censor: fn(&OsStr) -> &OsStr, inner: Box<dyn DynExecutor + Send>) -> Self {
        Self { censor, inner }
    }
}

impl Executor for CensoringExecutor {
    type Fut<'a> = BoxFuture<'a, Result<Output, ExecutorError>>;

    fn exec(&self, args: &[&OsStr]) -> Self::Fut<'_> {
        let mut censored = Vec::with_capacity(args.len());
        for arg in args {
            censored.push((self.censor)(arg.as_ref()));
        }

        self.inner.exec(&censored)
    }
}

pub struct GitUserInfo {
    pub name: String,
    pub email: String,
}

/// GitUserGetter fetches a name and email for us in git commits on-demand
pub trait GitUserGetter {
    type Fut: Future<Output = io::Result<GitUserInfo>> + Send;
    fn get(&self) -> Self::Fut;
}

pub trait DynGitUserGetter {
    fn get(&self) -> BoxFuture<io::Result<GitUserInfo>>;
}

impl<GG: GitUserGetter> DynGitUserGetter for GG {
    fn get(&self) -> BoxFuture<io::Result<GitUserInfo>> {
        Box::pin(GitUserGetter::get(self))
    }
}

pub struct DefaultGitUserGetter {}

impl GitUserGetter for DefaultGitUserGetter {
    type Fut = Ready<io::Result<GitUserInfo>>;

    fn get(&self) -> Self::Fut {
        ready(
            match (
                env::var("GIT_COMMITTER_NAME"),
                env::var("GIT_COMMITTER_EMAIL"),
            ) {
                (Ok(name), Ok(email)) => Ok(GitUserInfo { name, email }),
                _ => Err(io::ErrorKind::NotFound.into()),
            },
        )
    }
}

pub struct Git {
    dir: PathBuf,
    info: Box<dyn DynGitUserGetter + Send>,
    executor: Box<dyn DynExecutor + Send>,
}

impl Git {
    pub fn new(
        dir: impl Into<PathBuf>,
        info: Box<dyn DynGitUserGetter + Send>,
        executor: Box<dyn DynExecutor + Send>,
    ) -> Self {
        Self {
            dir: dir.into(),
            info,
            executor,
        }
    }

    /// Exposes the directory in which the repository has been cloned
    pub fn directory(&self) -> &Path {
        &self.dir
    }

    /// Clone clones the repository from a local path.
    pub async fn clone(&mut self, from: impl AsRef<OsStr>) -> Result<()> {
        tracing::info!(
            "Creating a clone of the repo at {} from {:?}",
            self.dir.display(),
            from.as_ref()
        );

        self.executor
            .exec(&[OsStr::new("clone"), from.as_ref(), self.dir.as_os_str()])
            .await
            .map(|_| ())
            .context("error creating a clone")
    }

    /// Stages all changes and commits them with the message
    pub async fn commit(
        &mut self,
        title: impl AsRef<OsStr>,
        body: impl AsRef<OsStr>,
    ) -> Result<()> {
        tracing::info!("Committing changes with title {:?}", title.as_ref());
        let author = self
            .info
            .get()
            .await
            .map(|info| format!("{} <{}>", info.name, info.email))
            .context("get git user info")?;

        let commands = [
            vec![OsStr::new("add"), OsStr::new("--all")],
            vec![
                OsStr::new("commit"),
                OsStr::new("--message"),
                title.as_ref(),
                OsStr::new("--message"),
                body.as_ref(),
                OsStr::new("--author"),
                OsStr::new(&author),
            ],
        ];
        for cmd in commands {
            self.executor
                .exec(&cmd)
                .await
                .with_context(|| format!("committing: {:?}", title.as_ref()))?;
        }
        Ok(())
    }

    /// Pushes the local state to the central remote
    pub async fn push(&mut self, remote: String, branch: String, force: bool) -> Result<()> {
        let mut args = vec![OsStr::new("push")];
        if force {
            args.push(OsStr::new("--force"));
        }
        args.extend(&[OsStr::new(&remote), OsStr::new(&branch)]);

        tracing::info!("Pushing branch {} to {}", branch, remote);
        self.executor
            .exec(&args)
            .await
            .with_context(|| format!("error pushing {}", branch))?;
        Ok(())
    }

    /// Removes the repository. It is up to the user to call this once they are done
    pub async fn clean(&mut self) -> Result<()> {
        fs::remove_dir_all(&self.dir).await?;
        Ok(())
    }

    // Tries to apply the patch in the given path into the current branch
    // by performing a three-way merge (similar to git cherry-pick). It returns
    // an error if the patch cannot be applied.
    pub async fn am(&mut self, path: impl AsRef<OsStr>) -> Result<()> {
        tracing::info!("Applying patch at {:?}", path.as_ref());
        if let Err(error) = self
            .executor
            .exec(&[OsStr::new("am"), OsStr::new("--3way"), path.as_ref()])
            .await
        {
            tracing::info!(error=?error, "Patch apply failed");
            if let ExecutorError::OutputError(_) = &error {
                let _ = self
                    .executor
                    .exec(&[OsStr::new("am"), OsStr::new("--abort")])
                    .await
                    .logit_warn("Aborting patch apply failed");
            }
            return Err(error.into());
        }

        Ok(())
    }

    /// Runs `git checkout`
    pub async fn checkout(&mut self, commitlike: impl AsRef<OsStr>) -> Result<()> {
        tracing::info!("Checking out {:?}", commitlike.as_ref());
        self.executor
            .exec(&[OsStr::new("checkout"), commitlike.as_ref()])
            .await
            .with_context(|| format!("error checking out {:?}", commitlike.as_ref()))
            .map(|_| ())
    }

    /// Creates a new branch and checks it out.
    pub async fn checkout_new_branch(&mut self, branch: impl AsRef<OsStr>) -> Result<()> {
        tracing::info!("Checking out new branch {:?}", branch.as_ref());

        self.executor
            .exec(&[OsStr::new("checkout"), OsStr::new("-b"), branch.as_ref()])
            .await
            .with_context(|| format!("error checking out new branch {:?}", branch.as_ref()))
            .map(|_| ())
    }

    /// Returns true if branch exists in heads.
    pub async fn branch_exists(&mut self, branch: impl AsRef<OsStr>) -> bool {
        tracing::info!("Checking if branch {:?} exists", branch.as_ref());
        self.executor
            .exec(&[
                OsStr::new("ls-remote"),
                OsStr::new("--exit-code"),
                OsStr::new("--heads"),
                OsStr::new("origin"),
                branch.as_ref(),
            ])
            .await
            .with_logit_warn(|| format!("error checking out new branch {:?}", branch.as_ref()))
            .is_ok()
    }

    /// Runs git config.
    pub async fn config(&mut self, args: &[impl AsRef<OsStr>]) -> Result<()> {
        let mut argsVec = Vec::with_capacity(args.len() + 1);
        argsVec.push(OsStr::new("config"));
        argsVec.extend(args.into_iter().map(AsRef::as_ref));
        tracing::info!(args=?argsVec, "Configuring.");

        self.executor
            .exec(&argsVec)
            .await
            .with_context(|| format!("error configuring {:?}", argsVec))
            .map(|_| ())
    }
}

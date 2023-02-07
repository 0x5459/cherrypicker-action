use futures::future::BoxFuture;
use octocrab::{models::Repository, Page};

use super::page::Pageable;

/// List repositories for a user
///
/// [See the GitHub API documentation](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
#[derive(serde::Serialize, Clone)]
pub struct ListReposForUserBuilder {
    username: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

impl ListReposForUserBuilder {
    fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            r#type: None,
            sort: None,
            direction: None,
            per_page: None,
            page: None,
        }
    }

    pub fn type_(mut self, r#type: impl Into<String>) -> Self {
        self.r#type = Some(r#type.into());
        self
    }

    /// One of `created` (when the repository was starred) or `updated` (when it was last pushed to).
    ///
    /// [See the GitHub API documentation](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// One of `asc` (ascending) or `desc` (descending).
    ///
    /// [See the GitHub API documentation](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
    pub fn direction(mut self, direction: impl Into<String>) -> Self {
        self.direction = Some(direction.into());
        self
    }

    /// Results per page (max 100).
    ///
    /// [See the GitHub API documentation](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
    pub fn per_page(mut self, per_page: impl Into<u8>) -> Self {
        self.per_page = Some(per_page.into());
        self
    }

    /// Page number of the results to fetch.
    ///
    /// [See the GitHub API documentation](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
    pub fn page(mut self, page: impl Into<u32>) -> Self {
        self.page = Some(page.into());
        self
    }

    /// Sends the actual request.
    pub async fn send(self) -> octocrab::Result<Page<Repository>> {
        let url = format!("users/{username}/repos", username = self.username,);
        octocrab::instance().get(url, Some(&self)).await
    }
}

/// List repositories for a user
///
/// [See github api doc](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
pub fn list_repos_for_user(username: impl Into<String>) -> ListReposForUserBuilder {
    ListReposForUserBuilder::new(username)
}

impl Pageable for ListReposForUserBuilder {
    type Item = Repository;

    type Error = octocrab::Error;

    type Fut = BoxFuture<'static, Result<Page<Self::Item>, Self::Error>>;

    fn list_by_page(&self, page: u32) -> Self::Fut {
        let this = self.clone();
        Box::pin(this.page(page).send())
    }
}

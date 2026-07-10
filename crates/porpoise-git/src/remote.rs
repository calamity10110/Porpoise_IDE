use async_trait::async_trait;
use octocrab::models::{issues::Issue as OctoIssue, pulls::PullRequest as OctoPr};
use porpoise_core::error::{PorpoiseError, Result};

use crate::types::{Issue, IssueState as OurIssueState, PrState, PullRequest};

#[async_trait]
pub trait RemoteProvider: Send + Sync {
    async fn list_prs(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>>;
    async fn get_pr(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest>;
    async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
    ) -> Result<PullRequest>;
    async fn merge_pr(&self, owner: &str, repo: &str, number: u64) -> Result<()>;
    async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>>;
}

pub struct GitHubProvider {
    client: octocrab::Octocrab,
}

impl GitHubProvider {
    pub fn new(token: Option<&str>) -> Result<Self> {
        let client = match token {
            Some(t) => octocrab::Octocrab::builder()
                .personal_token(t.to_string())
                .build()
                .map_err(|e| PorpoiseError::Git(format!("github auth: {e}")))?,
            None => octocrab::Octocrab::default(),
        };
        Ok(Self { client })
    }
}

fn pr_state(pr: &OctoPr) -> PrState {
    if pr.merged_at.is_some() {
        return PrState::Merged;
    }
    match pr.state {
        Some(octocrab::models::IssueState::Closed) => PrState::Closed,
        _ => PrState::Open,
    }
}

fn from_octo_pr(pr: OctoPr) -> PullRequest {
    let url = pr.html_url.clone().map(|u| u.to_string()).unwrap_or_default();
    let state = pr_state(&pr);
    let author = pr.user.map(|u| u.login).unwrap_or_default();
    let title = pr.title.unwrap_or_default();
    PullRequest {
        number: pr.number,
        title,
        author,
        body: pr.body,
        head_branch: pr.head.ref_field,
        base_branch: pr.base.ref_field,
        state,
        url,
    }
}

fn from_octo_issue(i: OctoIssue) -> Issue {
    Issue {
        number: i.number,
        title: i.title,
        author: i.user.login,
        body: i.body,
        state: match i.state {
            octocrab::models::IssueState::Closed => OurIssueState::Closed,
            _ => OurIssueState::Open,
        },
    }
}

#[async_trait]
impl RemoteProvider for GitHubProvider {
    async fn list_prs(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        let prs = self
            .client
            .pulls(owner, repo)
            .list()
            .per_page(50)
            .send()
            .await
            .map_err(|e| PorpoiseError::Git(format!("list PRs: {e}")))?;
        Ok(prs.items.into_iter().map(from_octo_pr).collect())
    }

    async fn get_pr(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest> {
        let pr = self
            .client
            .pulls(owner, repo)
            .get(number)
            .await
            .map_err(|e| PorpoiseError::Git(format!("get PR #{number}: {e}")))?;
        Ok(from_octo_pr(pr))
    }

    async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
    ) -> Result<PullRequest> {
        let pulls_handler = self.client.pulls(owner, repo);
        let mut create = pulls_handler.create(title, head, base);
        if let Some(b) = body {
            create = create.body(b);
        }
        let pr = create
            .send()
            .await
            .map_err(|e| PorpoiseError::Git(format!("create PR: {e}")))?;
        let mut result = from_octo_pr(pr);
        result.state = PrState::Open;
        Ok(result)
    }

    async fn merge_pr(&self, owner: &str, repo: &str, number: u64) -> Result<()> {
        self.client
            .pulls(owner, repo)
            .merge(number)
            .send()
            .await
            .map_err(|e| PorpoiseError::Git(format!("merge PR #{number}: {e}")))?;
        Ok(())
    }

    async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        let issues = self
            .client
            .issues(owner, repo)
            .list()
            .per_page(50)
            .send()
            .await
            .map_err(|e| PorpoiseError::Git(format!("list issues: {e}")))?;
        Ok(issues.items.into_iter().map(from_octo_issue).collect())
    }
}

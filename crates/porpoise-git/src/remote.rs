use async_trait::async_trait;
use octocrab::models::{issues::Issue as OctoIssue, pulls::PullRequest as OctoPr};
use porpoise_core::error::{PorpoiseError, Result};
use serde::{Deserialize, Serialize};

use crate::types::{Issue, IssueState as OurIssueState, PrState, PullRequest};

/// A pull request review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    pub id: u64,
    pub author: String,
    pub state: ReviewState,
    pub body: Option<String>,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Pending,
}

/// Commit status for a SHA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStatus {
    pub sha: String,
    pub state: StatusState,
    pub total_count: u32,
    pub statuses: Vec<StatusEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StatusState {
    Pending,
    Success,
    Failure,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    pub context: String,
    pub state: StatusState,
    pub description: Option<String>,
    pub target_url: Option<String>,
}

/// Branch protection rules summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtection {
    pub branch: String,
    pub required_reviews: u32,
    pub dismiss_stale_reviews: bool,
    pub require_code_owner_reviews: bool,
    pub enforce_admins: bool,
    pub required_status_checks: Vec<String>,
}

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

    // --- PR Reviews ---
    async fn list_reviews(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Vec<PrReview>>;
    async fn create_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
        event: ReviewState,
    ) -> Result<PrReview>;

    // --- Commit Status ---
    async fn get_combined_status(&self, owner: &str, repo: &str, sha: &str) -> Result<CommitStatus>;
    async fn create_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        state: StatusState,
        description: &str,
        context: &str,
        target_url: Option<&str>,
    ) -> Result<()>;

    // --- Branch Protection ---
    async fn get_branch_protection(&self, owner: &str, repo: &str, branch: &str) -> Result<BranchProtection>;
    async fn update_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        required_reviews: u32,
        dismiss_stale: bool,
    ) -> Result<()>;

    // --- Issue Linking ---
    async fn link_issue_to_pr(&self, owner: &str, repo: &str, pr_number: u64, issue_number: u64) -> Result<()>;
    async fn get_linked_issues(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Vec<Issue>>;
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

    async fn list_reviews(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Vec<PrReview>> {
        let route = format!("/repos/{owner}/{repo}/pulls/{pr_number}/reviews");
        let reviews: Vec<serde_json::Value> = self
            .client
            .get(route, None::<&()>)
            .await
            .map_err(|e| PorpoiseError::Git(format!("list reviews: {e}")))?;
        Ok(reviews
            .into_iter()
            .map(|r| PrReview {
                id: r["id"].as_u64().unwrap_or(0),
                author: r["user"]["login"].as_str().unwrap_or("").to_string(),
                state: match r["state"].as_str() {
                    Some("APPROVED") => ReviewState::Approved,
                    Some("CHANGES_REQUESTED") => ReviewState::ChangesRequested,
                    Some("COMMENTED") => ReviewState::Commented,
                    Some("DISMISSED") => ReviewState::Dismissed,
                    _ => ReviewState::Pending,
                },
                body: r["body"].as_str().map(String::from),
                submitted_at: r["submitted_at"].as_str().map(String::from),
            })
            .collect())
    }

    async fn create_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
        event: ReviewState,
    ) -> Result<PrReview> {
        let event_str = match event {
            ReviewState::Approved => "APPROVE",
            ReviewState::ChangesRequested => "REQUEST_CHANGES",
            _ => "COMMENT",
        };
        let route = format!("/repos/{owner}/{repo}/pulls/{pr_number}/reviews");
        let payload = serde_json::json!({ "body": body, "event": event_str });
        let r: serde_json::Value = self
            .client
            .post(route, Some(&payload))
            .await
            .map_err(|e| PorpoiseError::Git(format!("create review: {e}")))?;
        Ok(PrReview {
            id: r["id"].as_u64().unwrap_or(0),
            author: r["user"]["login"].as_str().unwrap_or("").to_string(),
            state: event,
            body: r["body"].as_str().map(String::from),
            submitted_at: r["submitted_at"].as_str().map(String::from),
        })
    }

    async fn get_combined_status(&self, owner: &str, repo: &str, sha: &str) -> Result<CommitStatus> {
        let route = format!("/repos/{owner}/{repo}/commits/{sha}/status");
        let data: serde_json::Value = self
            .client
            .get(route, None::<&()>)
            .await
            .map_err(|e| PorpoiseError::Git(format!("combined status: {e}")))?;
        let state = match data["state"].as_str() {
            Some("success") => StatusState::Success,
            Some("failure") => StatusState::Failure,
            Some("error") => StatusState::Error,
            _ => StatusState::Pending,
        };
        let statuses = data["statuses"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|s| StatusEntry {
                        context: s["context"].as_str().unwrap_or("").to_string(),
                        state: match s["state"].as_str() {
                            Some("success") => StatusState::Success,
                            Some("failure") => StatusState::Failure,
                            Some("error") => StatusState::Error,
                            _ => StatusState::Pending,
                        },
                        description: s["description"].as_str().map(String::from),
                        target_url: s["target_url"].as_str().map(String::from),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(CommitStatus {
            sha: sha.to_string(),
            state,
            total_count: data["total_count"].as_u64().unwrap_or(0) as u32,
            statuses,
        })
    }

    async fn create_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        state: StatusState,
        description: &str,
        context: &str,
        target_url: Option<&str>,
    ) -> Result<()> {
        let state_str = match state {
            StatusState::Pending => "pending",
            StatusState::Success => "success",
            StatusState::Failure => "failure",
            StatusState::Error => "error",
        };
        let route = format!("/repos/{owner}/{repo}/statuses/{sha}");
        let mut payload = serde_json::json!({
            "state": state_str,
            "description": description,
            "context": context,
        });
        if let Some(url) = target_url {
            payload["target_url"] = serde_json::json!(url);
        }
        self.client
            .post::<_, ()>(route, Some(&payload))
            .await
            .map_err(|e| PorpoiseError::Git(format!("create status: {e}")))?;
        Ok(())
    }

    async fn get_branch_protection(&self, owner: &str, repo: &str, branch: &str) -> Result<BranchProtection> {
        let route = format!("/repos/{owner}/{repo}/branches/{branch}/protection");
        let data: serde_json::Value = self
            .client
            .get(route, None::<&()>)
            .await
            .map_err(|e| PorpoiseError::Git(format!("branch protection: {e}")))?;
        let required_reviews = data["required_pull_request_reviews"]["required_approving_review_count"]
            .as_u64()
            .unwrap_or(1) as u32;
        let dismiss_stale = data["required_pull_request_reviews"]["dismiss_stale_reviews"]
            .as_bool()
            .unwrap_or(false);
        let require_code_owner = data["required_pull_request_reviews"]["require_code_owner_reviews"]
            .as_bool()
            .unwrap_or(false);
        let enforce_admins = data["enforce_admins"]["enabled"].as_bool().unwrap_or(false);
        let checks = data["required_status_checks"]["contexts"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        Ok(BranchProtection {
            branch: branch.to_string(),
            required_reviews,
            dismiss_stale_reviews: dismiss_stale,
            require_code_owner_reviews: require_code_owner,
            enforce_admins,
            required_status_checks: checks,
        })
    }

    async fn update_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        required_reviews: u32,
        dismiss_stale: bool,
    ) -> Result<()> {
        let route = format!("/repos/{owner}/{repo}/branches/{branch}/protection");
        let payload = serde_json::json!({
            "required_status_checks": null,
            "enforce_admins": false,
            "required_pull_request_reviews": {
                "required_approving_review_count": required_reviews,
                "dismiss_stale_reviews": dismiss_stale,
            },
            "restrictions": null,
        });
        self.client
            .put::<(), _, _>(route, Some(&payload))
            .await
            .map_err(|e| PorpoiseError::Git(format!("update protection: {e}")))?;
        Ok(())
    }

    async fn link_issue_to_pr(&self, owner: &str, repo: &str, pr_number: u64, issue_number: u64) -> Result<()> {
        // GitHub links issues via PR body keywords (e.g., "Closes #123") or timeline events.
        // We add a timeline comment to link them.
        let route = format!("/repos/{owner}/{repo}/issues/{pr_number}/comments");
        let payload = serde_json::json!({
            "body": format!("Linked to #{issue_number}")
        });
        self.client
            .post::<_, ()>(route, Some(&payload))
            .await
            .map_err(|e| PorpoiseError::Git(format!("link issue: {e}")))?;
        Ok(())
    }

    async fn get_linked_issues(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Vec<Issue>> {
        // Parse PR body + timeline events for issue references
        let pr = self.get_pr(owner, repo, pr_number).await?;
        let all_issues = self.list_issues(owner, repo).await?;
        let linked: Vec<Issue> = all_issues
            .into_iter()
            .filter(|issue| {
                let num = issue.number;
                let body = pr.body.as_deref().unwrap_or("");
                let body_lower = body.to_lowercase();
                // #N anywhere in body
                body_lower.contains(&format!("#{num}"))
                    // GitHub linking keywords (case-insensitive)
                    || body_lower.contains(&format!("closes #{num}"))
                    || body_lower.contains(&format!("close #{num}"))
                    || body_lower.contains(&format!("fixes #{num}"))
                    || body_lower.contains(&format!("fix #{num}"))
                    || body_lower.contains(&format!("resolves #{num}"))
                    || body_lower.contains(&format!("resolve #{num}"))
            })
            .collect();
        Ok(linked)
    }
}

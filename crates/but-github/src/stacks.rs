//! GitHub's native stacked-pull-requests REST API.
//!
//! Native stacks are enabled per repository. Their API models a stack as pull request numbers
//! ordered from the review closest to the target branch to the review at the top.

use std::collections::BTreeMap;

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};

use crate::client::GitHubClient;

/// A GitHub native pull-request stack.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Stack {
    /// The repository-scoped number used to address the stack.
    pub number: i64,
    /// Member pull requests ordered bottom-to-top.
    pub pull_requests: Vec<StackPullRequest>,
}

/// A pull request belonging to a native stack.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StackPullRequest {
    pub number: i64,
}

/// Whether the repository exposes GitHub's native stacks preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability<T> {
    Unsupported,
    Supported(T),
}

/// The native-stack membership mutation needed to reach a desired reviewed stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcilePlan {
    Noop,
    Create {
        desired: Vec<i64>,
    },
    Append {
        stack_number: i64,
        pull_requests: Vec<i64>,
    },
    Rebuild {
        stack_numbers: Vec<i64>,
        desired: Vec<i64>,
    },
    Dissolve {
        stack_numbers: Vec<i64>,
    },
}

impl ReconcilePlan {
    /// Native stack numbers that must be dissolved before PR targets are updated.
    pub fn stack_numbers_to_unstack(&self) -> &[i64] {
        match self {
            ReconcilePlan::Rebuild { stack_numbers, .. }
            | ReconcilePlan::Dissolve { stack_numbers } => stack_numbers,
            ReconcilePlan::Noop | ReconcilePlan::Create { .. } | ReconcilePlan::Append { .. } => {
                &[]
            }
        }
    }
}

/// Inspect native membership for every desired PR and calculate the required mutation.
///
/// Looking up every member is necessary when GitButler combines reviews that currently belong to
/// separate native stacks.
pub async fn prepare(
    preferred_account: Option<&crate::GithubAccountIdentifier>,
    owner: &str,
    repo: &str,
    desired: &[i64],
    storage: &but_forge_storage::Controller,
) -> Result<Availability<ReconcilePlan>> {
    let gh = GitHubClient::from_storage(storage, preferred_account)?;
    let Some(existing) = gh.existing_stacks(owner, repo, desired).await? else {
        return Ok(Availability::Unsupported);
    };
    Ok(Availability::Supported(reconcile_plan(existing, desired)))
}

/// Dissolve every native stack containing any of `pull_requests`.
pub async fn dissolve(
    preferred_account: Option<&crate::GithubAccountIdentifier>,
    owner: &str,
    repo: &str,
    pull_requests: &[i64],
    storage: &but_forge_storage::Controller,
) -> Result<Availability<()>> {
    let gh = GitHubClient::from_storage(storage, preferred_account)?;
    let Some(existing) = gh.existing_stacks(owner, repo, pull_requests).await? else {
        return Ok(Availability::Unsupported);
    };
    for stack in existing {
        gh.unstack(owner, repo, stack.number)
            .await
            .with_context(|| format!("Failed to dissolve GitHub stack #{}", stack.number))?;
    }
    Ok(Availability::Supported(()))
}

impl GitHubClient {
    async fn existing_stacks(
        &self,
        owner: &str,
        repo: &str,
        pull_requests: &[i64],
    ) -> Result<Option<Vec<Stack>>> {
        let mut existing = BTreeMap::new();
        for review_number in pull_requests {
            let Some(stacks) = self
                .stacks_for_pull_request(owner, repo, *review_number)
                .await
                .with_context(|| {
                    format!("Failed to inspect native stack membership for PR #{review_number}")
                })?
            else {
                return Ok(None);
            };
            for stack in stacks {
                existing.entry(stack.number).or_insert(stack);
            }
        }
        Ok(Some(existing.into_values().collect()))
    }
}

/// Dissolve native stacks that conflict with the desired stack shape.
pub async fn unstack_conflicting(
    preferred_account: Option<&crate::GithubAccountIdentifier>,
    owner: &str,
    repo: &str,
    plan: &ReconcilePlan,
    storage: &but_forge_storage::Controller,
) -> Result<()> {
    if plan.stack_numbers_to_unstack().is_empty() {
        return Ok(());
    }
    let gh = GitHubClient::from_storage(storage, preferred_account)?;
    for stack_number in plan.stack_numbers_to_unstack() {
        gh.unstack(owner, repo, *stack_number)
            .await
            .with_context(|| format!("Failed to dissolve GitHub stack #{stack_number}"))?;
    }
    Ok(())
}

/// Apply the create or append portion of a prepared plan after PR targets are correct.
pub async fn finish(
    preferred_account: Option<&crate::GithubAccountIdentifier>,
    owner: &str,
    repo: &str,
    plan: &ReconcilePlan,
    storage: &but_forge_storage::Controller,
) -> Result<()> {
    let gh = GitHubClient::from_storage(storage, preferred_account)?;
    match plan {
        ReconcilePlan::Noop | ReconcilePlan::Dissolve { .. } => Ok(()),
        ReconcilePlan::Create { desired } | ReconcilePlan::Rebuild { desired, .. } => gh
            .create_stack(owner, repo, desired)
            .await
            .context("Failed to create GitHub native stack"),
        ReconcilePlan::Append {
            stack_number,
            pull_requests,
        } => gh
            .add_to_stack(owner, repo, *stack_number, pull_requests)
            .await
            .with_context(|| format!("Failed to extend GitHub stack #{stack_number}")),
    }
}

fn reconcile_plan(mut existing: Vec<Stack>, desired: &[i64]) -> ReconcilePlan {
    existing.sort_by_key(|stack| stack.number);
    let stack_numbers = existing
        .iter()
        .map(|stack| stack.number)
        .collect::<Vec<_>>();
    if desired.len() < 2 {
        return if stack_numbers.is_empty() {
            ReconcilePlan::Noop
        } else {
            ReconcilePlan::Dissolve { stack_numbers }
        };
    }
    if existing.is_empty() {
        return ReconcilePlan::Create {
            desired: desired.to_vec(),
        };
    }
    if existing.len() == 1 {
        let stack = &existing[0];
        let members = stack
            .pull_requests
            .iter()
            .map(|review| review.number)
            .collect::<Vec<_>>();
        if members == desired {
            return ReconcilePlan::Noop;
        }
        if desired.starts_with(&members) {
            return ReconcilePlan::Append {
                stack_number: stack.number,
                pull_requests: desired[members.len()..].to_vec(),
            };
        }
    }
    ReconcilePlan::Rebuild {
        stack_numbers,
        desired: desired.to_vec(),
    }
}

#[derive(Serialize)]
struct StackMembersBody<'a> {
    pull_requests: &'a [i64],
}

impl GitHubClient {
    /// `Ok(None)` means the repository does not expose the native stacks endpoint.
    async fn stacks_for_pull_request(
        &self,
        owner: &str,
        repo: &str,
        review_number: i64,
    ) -> Result<Option<Vec<Stack>>> {
        let response = self
            .client
            .get(format!("{}/repos/{owner}/{repo}/stacks", self.base_url))
            .query(&[("pull_request", review_number)])
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            bail!(
                "Failed to list GitHub stacks: {}",
                response_error(response).await
            );
        }
        Ok(Some(response.json().await?))
    }

    async fn create_stack(&self, owner: &str, repo: &str, desired: &[i64]) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/repos/{owner}/{repo}/stacks", self.base_url))
            .json(&StackMembersBody {
                pull_requests: desired,
            })
            .send()
            .await?;
        if !response.status().is_success() {
            bail!(
                "Failed to create GitHub stack: {}",
                response_error(response).await
            );
        }
        Ok(())
    }

    async fn add_to_stack(
        &self,
        owner: &str,
        repo: &str,
        stack_number: i64,
        pull_requests: &[i64],
    ) -> Result<()> {
        let response = self
            .client
            .post(format!(
                "{}/repos/{owner}/{repo}/stacks/{stack_number}/add",
                self.base_url
            ))
            .json(&StackMembersBody { pull_requests })
            .send()
            .await?;
        if !response.status().is_success() {
            bail!(
                "Failed to add to GitHub stack: {}",
                response_error(response).await
            );
        }
        Ok(())
    }

    async fn unstack(&self, owner: &str, repo: &str, stack_number: i64) -> Result<()> {
        let response = self
            .client
            .post(format!(
                "{}/repos/{owner}/{repo}/stacks/{stack_number}/unstack",
                self.base_url
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            bail!(
                "Failed to unstack GitHub stack: {}",
                response_error(response).await
            );
        }
        Ok(())
    }
}

async fn response_error(response: reqwest::Response) -> String {
    let status = response.status();
    match response.text().await {
        Ok(body) if !body.is_empty() => format!("{status}: {body}"),
        _ => status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(number: i64, members: &[i64]) -> Stack {
        Stack {
            number,
            pull_requests: members
                .iter()
                .map(|number| StackPullRequest { number: *number })
                .collect(),
        }
    }

    #[test]
    fn plans_creation_for_an_unregistered_stack() {
        assert_eq!(
            reconcile_plan(Vec::new(), &[1, 2, 3]),
            ReconcilePlan::Create {
                desired: vec![1, 2, 3]
            }
        );
    }

    #[test]
    fn exact_membership_is_a_noop() {
        assert_eq!(
            reconcile_plan(vec![stack(7, &[1, 2, 3])], &[1, 2, 3]),
            ReconcilePlan::Noop
        );
    }

    #[test]
    fn only_a_top_suffix_can_be_appended() {
        assert_eq!(
            reconcile_plan(vec![stack(7, &[1, 2])], &[1, 2, 3, 4]),
            ReconcilePlan::Append {
                stack_number: 7,
                pull_requests: vec![3, 4]
            }
        );
    }

    #[test]
    fn insertion_removal_and_reorder_rebuild_the_stack() {
        for (existing, desired) in [
            (vec![stack(7, &[1, 3])], vec![1, 2, 3]),
            (vec![stack(7, &[1, 2, 3])], vec![1, 3]),
            (vec![stack(7, &[1, 2, 3])], vec![3, 2, 1]),
        ] {
            assert_eq!(
                reconcile_plan(existing, &desired),
                ReconcilePlan::Rebuild {
                    stack_numbers: vec![7],
                    desired
                }
            );
        }
    }

    #[test]
    fn combining_native_stacks_dissolves_all_of_them() {
        assert_eq!(
            reconcile_plan(vec![stack(7, &[1, 2]), stack(9, &[3, 4])], &[1, 2, 3, 4]),
            ReconcilePlan::Rebuild {
                stack_numbers: vec![7, 9],
                desired: vec![1, 2, 3, 4]
            }
        );
    }

    #[test]
    fn singleton_reviews_are_not_native_stacks() {
        assert_eq!(
            reconcile_plan(vec![stack(7, &[1, 2])], &[1]),
            ReconcilePlan::Dissolve {
                stack_numbers: vec![7]
            }
        );
        assert_eq!(reconcile_plan(Vec::new(), &[1]), ReconcilePlan::Noop);
    }
}

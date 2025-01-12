use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use serde::Deserialize;
use std::env;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<Issue>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    number: u64,
    pull_request: Option<PullRequestRef>,
}

#[derive(Debug, Deserialize)]
struct PullRequestRef {}

const NUMBER_OF_EXPERIMENTAL_ENVIRONMENTS: usize = 8;

async fn fetch_prs(
    octocrab: Arc<octocrab::Octocrab>,
    owner: String,
    repo: String,
    current_user: String,
) -> Result<Vec<octocrab::models::pulls::PullRequest>> {
    // Fetch PRs logic moved to separate function
    let search_response = octocrab
        .get::<SearchResponse, _, _>(
            "/search/issues",
            Some(&serde_json::json!({
                "q": format!("type:pr state:open author:{} repo:{}/{}", current_user, owner, repo)
            })),
        )
        .await
        .context("Failed to fetch PRs. Please check repository name and permissions")?;

    let mut prs = Vec::new();
    for issue in search_response.items {
        if let Some(_pr_ref) = issue.pull_request {
            if let Some(pr) = octocrab.pulls(&owner, &repo).get(issue.number).await.ok() {
                prs.push(pr);
            }
        }
    }
    Ok(prs)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // Get GitHub token from environment
    let token = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not found in environment")?;

    let workflow_id = env::var("DEPLOY_EXPERIMENTAL_WORKFLOW_ID")
        .context("DEPLOY_EXPERIMENTAL_WORKFLOW_ID not found in environment")?;

    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()?;

    // Get organization and repo from environment
    let owner = env::var("GITHUB_ORG").context("GITHUB_ORG not found in environment")?;
    let repo = env::var("GITHUB_REPO").context("GITHUB_REPO not found in environment")?;

    println!("Authenticating with GitHub...");

    // Get current user's login
    println!("Fetching current user info...");
    let current_user = octocrab
        .current()
        .user()
        .await
        .context(
            "Failed to fetch current user. Please check your GitHub token has correct permissions",
        )?
        .login;
    println!("Authenticated as: {}", current_user);

    let octocrab = Arc::new(octocrab);
    let pr_fetch = tokio::spawn({
        let octocrab = Arc::clone(&octocrab);
        let owner = owner.clone();
        let repo = repo.clone();
        let current_user = current_user.clone();
        async move { fetch_prs(octocrab, owner, repo, current_user).await }
    });

    // Show environment selection while PRs are being fetched
    let environments: Vec<String> = (1..=NUMBER_OF_EXPERIMENTAL_ENVIRONMENTS)
        .map(|i| format!("experimental{i}"))
        .collect();
    let env_options: Vec<String> = environments.iter().map(|e| e.to_string()).collect();

    let env_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select environment to use")
        .items(&env_options)
        .default(0)
        .interact()?;

    let selected_env = &environments[env_selection];

    // Wait for PR fetching to complete
    println!("Fetching PRs from {}/{}...", owner, repo);
    let prs = pr_fetch.await.context("PR fetch task failed")??;

    // Rest of the PR selection and workflow dispatch code remains the same
    let pr_titles: Vec<String> = prs
        .iter()
        .map(|pr| {
            format!(
                "#{} - {}",
                pr.number,
                pr.title.as_ref().unwrap_or(&String::new())
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a PR")
        .items(&pr_titles)
        .default(0)
        .interact()?;

    let selected_pr = &prs[selection];
    let branch_name = selected_pr.head.ref_field.clone();

    // Get the last commit from the branch
    let commits = octocrab
        .repos(&owner, &repo)
        .list_commits()
        .branch(&branch_name)
        .send()
        .await?;

    let last_commit = commits
        .items
        .first()
        .context("No commits found in branch")?;

    let commit_hash = last_commit.sha[..7].to_string();

    // Trigger the GitHub Action using the proper workflow ID
    let body = serde_json::json!({
        "ref": branch_name,
        "inputs": {
            "commit_sha": commit_hash,
            "target": format!("{}",  selected_env)
        }
    });

    // Trigger the GitHub Action using the proper workflow ID
    octocrab
        .actions()
        .create_workflow_dispatch(
            &owner,
            &repo,
            workflow_id, // selected_workflow.id.to_string(),
            &branch_name,
        )
        .inputs(serde_json::Value::Object(
            body.as_object()
                .unwrap()
                .get("inputs")
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        ))
        .send()
        .await
        .context(
            "Failed to trigger workflow. Please check workflow inputs match your workflow file.",
        )?;

    println!("Successfully triggered GitHub Action:");
    println!("Branch: {}", branch_name);
    println!("Commit: {}", commit_hash);
    println!("Environment: {}", selected_env);

    Ok(())
}

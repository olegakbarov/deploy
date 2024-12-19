use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use octocrab::{models::pulls::PullRequest, params::State};
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Environment number (1-6)
    #[arg(value_parser = validate_environment)]
    environment: u8,
}

fn validate_environment(s: &str) -> Result<u8, String> {
    let env: u8 = s.parse().map_err(|_| "Environment must be a number")?;
    if (1..=6).contains(&env) {
        Ok(env)
    } else {
        Err("Environment must be between 1 and 6".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();

    // Get GitHub token from environment
    let token = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not found in environment")?;

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

    println!("Fetching PRs from {}/{}...", owner, repo);
    // Get open PRs
    let pulls = octocrab
        .pulls(&owner, &repo)
        .list()
        .state(State::Open)
        .send()
        .await
        .context("Failed to fetch PRs. Please check repository name and permissions")?;

    // Filter PRs by author
    let prs: Vec<PullRequest> = pulls
        .items
        .into_iter()
        .filter(|pr| {
            pr.user
                .as_ref()
                .map(|u| u.login == current_user)
                .unwrap_or(false)
        })
        .collect();

    if prs.is_empty() {
        println!("No open pull requests found for your user");
        return Ok(());
    }

    // Create selection menu for PRs
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

    println!("Branch: {}", branch_name);
    println!("Commit: {}", commit_hash);
    println!("Environment: experimental{}", args.environment);

    // List available workflows first
    let workflows = octocrab
        .workflows(&owner, &repo)
        .list()
        .send()
        .await
        .context("Failed to fetch workflows")?;

    println!("\nAvailable workflows:");
    for workflow in &workflows.items {
        println!(
            "ID: {}, Name: {}, File: {}",
            workflow.id, workflow.name, workflow.path
        );
    }

    // Ask user to select workflow
    let workflow_options: Vec<String> = workflows
        .items
        .iter()
        .map(|w| format!("{} ({})", w.name, w.path))
        .collect();

    let workflow_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select workflow to run")
        .items(&workflow_options)
        .default(0)
        .interact()?;

    let selected_workflow = &workflows.items[workflow_selection];

    println!(
        "\nTriggering workflow: {} (ID: {})",
        selected_workflow.name, selected_workflow.id
    );

    // Trigger the GitHub Action using the proper workflow ID
    let body = serde_json::json!({
        "ref": branch_name,
        "inputs": {
            "commit_sha": commit_hash,
            "target": format!("experimental{}", args.environment)
        }
    });

    println!(
        "Sending request with payload: {}",
        serde_json::to_string_pretty(&body)?
    );

    // Trigger the GitHub Action using the proper workflow ID
    octocrab
        .actions()
        .create_workflow_dispatch(
            &owner,
            &repo,
            selected_workflow.id.to_string(),
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
    println!("Environment: experimental{}", args.environment);

    Ok(())
}

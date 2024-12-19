# GitHub Actions Workflow Trigger CLI

A command-line tool for triggering GitHub Actions workflows for your pull requests in experimental environments.

## Features

- Authentication with GitHub using personal access token
- Lists and filters your open pull requests
- Interactive selection of pull requests and workflows
- Triggers workflows with specific commit and environment parameters
- Supports multiple experimental environments (1-6)

## Prerequisites

- Rust toolchain installed
- GitHub personal access token with appropriate permissions
- Access to the target GitHub organization and repository

## Installation

1. Clone the repository
2. Build the project:
```bash
cargo build --release
```
3. The binary will be available in `target/release/`

## Configuration

Create a `.env` file in the project root with the following variables:

```env
GITHUB_TOKEN=your_personal_access_token
GITHUB_ORG=your_organization_name
GITHUB_REPO=your_repository_name
```

### Required GitHub Token Permissions

Your personal access token needs the following permissions:
- `repo` access to read repository information and create workflow dispatches
- `workflow` access to trigger workflows

## Usage

```bash
# Run workflow for experimental environment 1
workflow-trigger 1

# Run workflow for experimental environment 2
workflow-trigger 2
```

The tool will:
1. Authenticate with GitHub
2. Fetch your open pull requests
3. Present an interactive menu to select a PR
4. Display available workflows
5. Let you select a workflow to trigger
6. Trigger the selected workflow with the appropriate parameters

## Arguments

- `environment`: Required. A number between 1 and 6 specifying the experimental environment.

## Workflow Input Parameters

The tool sends the following parameters to the workflow:
- `commit_sha`: The first 7 characters of the latest commit hash
- `target`: The target environment in the format `experimental{N}`

## Error Handling

The tool includes comprehensive error handling for common scenarios:
- Invalid environment numbers
- Missing environment variables
- Authentication failures
- API permission issues
- Missing workflows or repositories

## Development

This project uses the following dependencies:
- `clap` for argument parsing
- `octocrab` for GitHub API interaction
- `dialoguer` for interactive prompts
- `anyhow` for error handling
- `dotenv` for environment variable management
- `tokio` for async runtime

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

GNU General Public License (GPL)

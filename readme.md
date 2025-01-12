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
cp target/release/deploy ~/.local/bin/deploy
```

```bash
chmod +x ~/.local/bin/deploy
```

```bash
# in `.zshrc`:
export GITHUB_TOKEN="your_github_personal_access_token"
export GITHUB_ORG="your_org_name"
export GITHUB_REPO="your_repo_name"
export DEPLOY_EXPERIMENTAL_WORKFLOW_ID="your_workflow_id"
```

## Developing locally Configuration

Create a `.env` file in the project root with the following variables:

```env
GITHUB_TOKEN=your_personal_access_token
GITHUB_ORG=your_organization_name
GITHUB_REPO=your_repository_name
DEPLOY_EXPERIMENTAL_WORKFLOW_ID=your_workflow_id
```

### Required GitHub Token Permissions

Your personal access token needs the following permissions:
- `repo` access to read repository information and create workflow dispatches
- `workflow` access to trigger workflows


## Usage

```bash
deploy
```

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

GNU General Public License (GPL)

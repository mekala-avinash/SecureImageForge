terraform {
  required_version = ">= 1.5"
  required_providers {
    github = {
      source  = "integrations/github"
      version = "~> 6.2"
    }
  }
}

# Provider expects GITHUB_TOKEN env var with admin:org scope.
provider "github" {
  owner = var.org
}

variable "org" {
  type        = string
  description = "GitHub organization (e.g. acme)"
}

variable "repos" {
  type        = list(string)
  description = "Repos to protect (must already exist)"
  default     = ["gitops", "platform", "tenants"]
}

variable "required_reviewers" {
  type    = number
  default = 2
}

variable "required_status_checks" {
  type = list(string)
  default = [
    "lint",
    "build",
    "kubeconform",
    "policy",
    "secret-scan",
  ]
}

# -----------------------------------------------------------------------------
# Branch protection on `main`
# -----------------------------------------------------------------------------
resource "github_branch_protection" "main" {
  for_each      = toset(var.repos)
  repository_id = each.value
  pattern       = "main"

  enforce_admins         = true
  allows_deletions       = false
  allows_force_pushes    = false
  require_signed_commits = true
  required_linear_history = true
  require_conversation_resolution = true

  required_pull_request_reviews {
    required_approving_review_count = var.required_reviewers
    dismiss_stale_reviews           = true
    require_code_owner_reviews      = true
    require_last_push_approval      = true
  }

  required_status_checks {
    strict   = true
    contexts = var.required_status_checks
  }

  restrict_pushes {
    # Only allow the Argo CD bot and platform-admins team to push directly to main.
    push_allowances = [
      "/${var.org}/platform-admins",
      "${var.org}-argocd-bot",
    ]
  }
}

# -----------------------------------------------------------------------------
# Repo-level settings
# -----------------------------------------------------------------------------
resource "github_repository" "settings" {
  for_each = toset(var.repos)
  name     = each.value

  visibility                  = "private"
  has_issues                  = true
  has_projects                = false
  has_wiki                    = false
  allow_merge_commit          = false
  allow_squash_merge          = true
  allow_rebase_merge          = false
  allow_auto_merge            = true
  delete_branch_on_merge      = true
  vulnerability_alerts        = true
  web_commit_signoff_required = true

  security_and_analysis {
    secret_scanning              { status = "enabled" }
    secret_scanning_push_protection { status = "enabled" }
  }

  lifecycle {
    # Tolerate the repo already existing (created via `gh repo create`).
    ignore_changes = [description, topics]
  }
}

# -----------------------------------------------------------------------------
# Outputs
# -----------------------------------------------------------------------------
output "protected_repos" {
  value = [for r in var.repos : "${var.org}/${r}"]
}

output "required_status_checks" {
  value = var.required_status_checks
}

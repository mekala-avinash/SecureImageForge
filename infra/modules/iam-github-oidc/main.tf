# GitHub OIDC → AWS IAM federation for ACME platform CI.
#
# Replaces long-lived `ECR_PASS` and `GITOPS_TOKEN` secrets with short-lived
# AWS STS tokens issued on-demand to GitHub Actions workflows.
#
# Usage from a service repo's reusable build workflow:
#   permissions:
#     id-token: write
#   - uses: aws-actions/configure-aws-credentials@v4
#     with:
#       role-to-assume: arn:aws:iam::${{ vars.AWS_ACCOUNT }}:role/gha-build
#       aws-region: ${{ vars.AWS_REGION }}

terraform {
  required_version = ">= 1.6.0"
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.70" }
  }
}

# ── 1. OIDC provider ─────────────────────────────────────────────────────────
resource "aws_iam_openid_connect_provider" "github" {
  url             = "https://token.actions.githubusercontent.com"
  client_id_list  = ["sts.amazonaws.com"]
  # GitHub rotates these — `aws iam list-open-id-connect-providers` to refresh.
  thumbprint_list = [
    "6938fd4d98bab03faadb97b34396831e3780aea1",
    "1c58a3a8518e8759bf075b76b750d4f2df264fcd",
  ]
  tags = var.tags
}

# ── 2. Trust policy ─────────────────────────────────────────────────────────
data "aws_iam_policy_document" "trust" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRoleWithWebIdentity"]
    principals {
      type        = "Federated"
      identifiers = [aws_iam_openid_connect_provider.github.arn]
    }
    condition {
      test     = "StringEquals"
      variable = "token.actions.githubusercontent.com:aud"
      values   = ["sts.amazonaws.com"]
    }
    condition {
      test     = "StringLike"
      variable = "token.actions.githubusercontent.com:sub"
      # Scope to specific org + repo set + ref pattern. Pin in production.
      values = [for r in var.allowed_repos : "repo:${r}:*"]
    }
  }
}

# ── 3. Role assumed by gha workflows ────────────────────────────────────────
resource "aws_iam_role" "gha_build" {
  name                 = "gha-build"
  assume_role_policy   = data.aws_iam_policy_document.trust.json
  max_session_duration = 3600
  tags                 = var.tags
}

# ── 4. Inline policy: ECR push + KMS decrypt for signed images ──────────────
data "aws_iam_policy_document" "ecr_push" {
  statement {
    sid     = "EcrAuth"
    effect  = "Allow"
    actions = ["ecr:GetAuthorizationToken"]
    resources = ["*"]
  }
  statement {
    sid    = "EcrPush"
    effect = "Allow"
    actions = [
      "ecr:BatchCheckLayerAvailability",
      "ecr:CompleteLayerUpload",
      "ecr:DescribeImages",
      "ecr:DescribeRepositories",
      "ecr:GetDownloadUrlForLayer",
      "ecr:InitiateLayerUpload",
      "ecr:PutImage",
      "ecr:UploadLayerPart",
      "ecr:TagResource",
    ]
    resources = [
      "arn:aws:ecr:${var.region}:${var.account_id}:repository/*",
    ]
  }
  dynamic "statement" {
    for_each = var.kms_key_arn == null ? [] : [1]
    content {
      sid     = "KmsForSignedImages"
      effect  = "Allow"
      actions = ["kms:Decrypt", "kms:Encrypt", "kms:GenerateDataKey"]
      resources = [var.kms_key_arn]
    }
  }
}

resource "aws_iam_role_policy" "ecr_push" {
  name   = "ecr-push"
  role   = aws_iam_role.gha_build.id
  policy = data.aws_iam_policy_document.ecr_push.json
}

# ── 5. Outputs ───────────────────────────────────────────────────────────────
output "role_arn"      { value = aws_iam_role.gha_build.arn }
output "provider_arn"  { value = aws_iam_openid_connect_provider.github.arn }

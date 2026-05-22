# aws-irsa — IAM Role for Service Account.
# Generic, least-privilege IRSA module callable by any platform component or workload.
terraform {
  required_version = ">= 1.5"
  required_providers { aws = { source = "hashicorp/aws", version = "~> 5.70" } }
}

variable "role_name"         { type = string }
variable "oidc_provider_arn" { type = string }
variable "oidc_provider_url" { type = string }
variable "namespace"         { type = string }
variable "service_account"   { type = string }
variable "policy_arns"       { type = list(string) default = [] }
variable "inline_policy_json"{ type = string  default = "" }
variable "tags"              { type = map(string) default = {} }

data "aws_iam_policy_document" "assume" {
  statement {
    actions = ["sts:AssumeRoleWithWebIdentity"]
    effect  = "Allow"
    principals { type = "Federated" identifiers = [var.oidc_provider_arn] }
    condition {
      test     = "StringEquals"
      variable = "${replace(var.oidc_provider_url, "https://", "")}:sub"
      values   = ["system:serviceaccount:${var.namespace}:${var.service_account}"]
    }
    condition {
      test     = "StringEquals"
      variable = "${replace(var.oidc_provider_url, "https://", "")}:aud"
      values   = ["sts.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "this" {
  name               = var.role_name
  assume_role_policy = data.aws_iam_policy_document.assume.json
  tags               = var.tags
}

resource "aws_iam_role_policy_attachment" "managed" {
  for_each   = toset(var.policy_arns)
  role       = aws_iam_role.this.name
  policy_arn = each.value
}

resource "aws_iam_role_policy" "inline" {
  count  = var.inline_policy_json == "" ? 0 : 1
  name   = "${var.role_name}-inline"
  role   = aws_iam_role.this.id
  policy = var.inline_policy_json
}

output "role_arn"  { value = aws_iam_role.this.arn }
output "role_name" { value = aws_iam_role.this.name }
output "annotation_value" {
  description = "Drop this into a Kubernetes ServiceAccount annotation: eks.amazonaws.com/role-arn"
  value       = aws_iam_role.this.arn
}

variable "region" {
  type        = string
  description = "AWS region (used in ECR ARNs)."
}

variable "account_id" {
  type        = string
  description = "AWS account id (12-digit string)."
}

variable "allowed_repos" {
  type        = list(string)
  description = "GitHub repos allowed to assume gha-build (e.g. [\"acme/platform\", \"acme/orders-api\"])."
  default     = ["acme/platform", "acme/gitops"]
  validation {
    condition     = length(var.allowed_repos) > 0
    error_message = "Specify at least one repository."
  }
}

variable "kms_key_arn" {
  type        = string
  description = "Optional KMS key for image signing/encryption. Set null to omit."
  default     = null
}

variable "tags" {
  type    = map(string)
  default = { "paved-road" = "true", "managed-by" = "terraform" }
}

# aws-kms-bucket — KMS-encrypted S3 bucket with Object Lock, lifecycle, public-access-block.
# Used for SLSA provenance archives, Loki/Tempo storage, Velero backups, audit-log lake.
terraform {
  required_version = ">= 1.5"
  required_providers { aws = { source = "hashicorp/aws", version = "~> 5.70" } }
}

variable "name"                 { type = string }
variable "object_lock_enabled"  { type = bool   default = false }
variable "object_lock_mode"     { type = string default = "COMPLIANCE" }
variable "object_lock_days"     { type = number default = 2555 } # 7 years
variable "lifecycle_transitions" {
  type = list(object({ days = number, storage_class = string }))
  default = [
    { days = 30,  storage_class = "STANDARD_IA" },
    { days = 90,  storage_class = "GLACIER_IR" },
    { days = 365, storage_class = "DEEP_ARCHIVE" },
  ]
}
variable "tags" { type = map(string) default = {} }

resource "aws_kms_key" "this" {
  description             = "KMS CMK for s3://${var.name}"
  deletion_window_in_days = 30
  enable_key_rotation     = true
  tags                    = var.tags
}

resource "aws_kms_alias" "this" { name = "alias/${var.name}" target_key_id = aws_kms_key.this.id }

resource "aws_s3_bucket" "this" {
  bucket              = var.name
  object_lock_enabled = var.object_lock_enabled
  tags                = var.tags
}

resource "aws_s3_bucket_versioning" "this" {
  bucket = aws_s3_bucket.this.id
  versioning_configuration { status = "Enabled" }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "this" {
  bucket = aws_s3_bucket.this.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm     = "aws:kms"
      kms_master_key_id = aws_kms_key.this.arn
    }
    bucket_key_enabled = true
  }
}

resource "aws_s3_bucket_public_access_block" "this" {
  bucket                  = aws_s3_bucket.this.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_object_lock_configuration" "this" {
  count  = var.object_lock_enabled ? 1 : 0
  bucket = aws_s3_bucket.this.id
  rule {
    default_retention {
      mode = var.object_lock_mode
      days = var.object_lock_days
    }
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "this" {
  bucket = aws_s3_bucket.this.id
  rule {
    id     = "tiered-storage"
    status = "Enabled"
    dynamic "transition" {
      for_each = var.lifecycle_transitions
      content {
        days          = transition.value.days
        storage_class = transition.value.storage_class
      }
    }
    noncurrent_version_expiration { noncurrent_days = 90 }
  }
}

output "bucket_arn" { value = aws_s3_bucket.this.arn }
output "bucket_id"  { value = aws_s3_bucket.this.id }
output "kms_arn"    { value = aws_kms_key.this.arn }

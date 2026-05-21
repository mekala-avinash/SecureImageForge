# eks-terraform-module.tf — prod-use1 single-region cluster
# Lives in acme/platform repo: modules/aws-eks-prod-cluster/
# Consumed from acme/gitops repo: gitops/clusters/prod-use1/cluster.tf

terraform {
  required_version = ">= 1.5"
  required_providers {
    aws  = { source = "hashicorp/aws",  version = "~> 5.70" }
    helm = { source = "hashicorp/helm", version = "~> 2.15" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.32" }
  }
}

variable "cluster_name"     { type = string  default = "prod-use1" }
variable "region"           { type = string  default = "us-east-1" }
variable "kubernetes_version" { type = string  default = "1.31" }
variable "vpc_cidr"         { type = string  default = "10.20.0.0/16" }
variable "azs"              { type = list(string) default = ["us-east-1a","us-east-1b","us-east-1c"] }
variable "tags"             { type = map(string) default = { Program = "EPT", Env = "prod" } }

# -----------------------------------------------------------------------------
# VPC (use upstream module for hardened defaults)
# -----------------------------------------------------------------------------
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.13"

  name = var.cluster_name
  cidr = var.vpc_cidr
  azs  = var.azs

  private_subnets = [for i in range(length(var.azs)) : cidrsubnet(var.vpc_cidr, 3, i)]
  public_subnets  = [for i in range(length(var.azs)) : cidrsubnet(var.vpc_cidr, 6, i + 24)]

  enable_nat_gateway     = true
  single_nat_gateway     = false
  one_nat_gateway_per_az = true
  enable_vpn_gateway     = false

  enable_flow_log                      = true
  create_flow_log_cloudwatch_iam_role  = true
  create_flow_log_cloudwatch_log_group = true
  flow_log_max_aggregation_interval    = 60

  private_subnet_tags = {
    "kubernetes.io/role/internal-elb" = "1"
    "karpenter.sh/discovery"          = var.cluster_name
  }
  public_subnet_tags = { "kubernetes.io/role/elb" = "1" }
  tags = var.tags
}

# -----------------------------------------------------------------------------
# KMS CMK for secrets encryption at rest
# -----------------------------------------------------------------------------
resource "aws_kms_key" "eks" {
  description             = "EKS secrets encryption — ${var.cluster_name}"
  deletion_window_in_days = 30
  enable_key_rotation     = true
  tags                    = var.tags
}

# -----------------------------------------------------------------------------
# EKS cluster (upstream module, hardened defaults)
# -----------------------------------------------------------------------------
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.24"

  cluster_name    = var.cluster_name
  cluster_version = var.kubernetes_version

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  cluster_endpoint_public_access  = false   # private only
  cluster_endpoint_private_access = true

  cluster_encryption_config = {
    provider_key_arn = aws_kms_key.eks.arn
    resources        = ["secrets"]
  }

  cluster_enabled_log_types = ["api", "audit", "authenticator", "controllerManager", "scheduler"]

  authentication_mode = "API"   # no aws-auth ConfigMap

  cluster_addons = {
    coredns                = { most_recent = true }
    kube-proxy             = { most_recent = true, configuration_values = jsonencode({ mode = "none" }) } # Cilium kube-proxy replacement
    eks-pod-identity-agent = { most_recent = true }
    aws-ebs-csi-driver     = { most_recent = true }
    aws-efs-csi-driver     = { most_recent = true }
    snapshot-controller    = { most_recent = true }
    # vpc-cni intentionally omitted — Cilium replaces it
  }

  # Minimal "system" managed node group for cluster-critical add-ons that can't run on Karpenter (chicken/egg)
  eks_managed_node_groups = {
    system = {
      ami_type       = "BOTTLEROCKET_x86_64"
      instance_types = ["m6i.large"]
      min_size       = 3
      max_size       = 6
      desired_size   = 3
      labels         = { workload = "system" }
      taints         = [{ key = "CriticalAddonsOnly", value = "true", effect = "NO_SCHEDULE" }]
      iam_role_additional_policies = {
        ssm = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
      }
      block_device_mappings = {
        xvda = { device_name = "/dev/xvda", ebs = { volume_type = "gp3", volume_size = 100, encrypted = true, kms_key_id = aws_kms_key.eks.arn } }
      }
    }
  }

  # Karpenter discovery
  node_security_group_tags = { "karpenter.sh/discovery" = var.cluster_name }

  tags = var.tags
}

# -----------------------------------------------------------------------------
# IRSA roles for platform components (excerpt — full set in module)
# -----------------------------------------------------------------------------
module "karpenter" {
  source  = "terraform-aws-modules/eks/aws//modules/karpenter"
  version = "~> 20.24"

  cluster_name          = module.eks.cluster_name
  enable_v1_permissions = true
  enable_pod_identity   = true
  create_pod_identity_association = true
  node_iam_role_additional_policies = {
    ssm = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
  }
  tags = var.tags
}

# -----------------------------------------------------------------------------
# OIDC trust for IRSA-based platform components (Loki/Tempo S3, etc.)
# -----------------------------------------------------------------------------
data "aws_iam_policy_document" "loki_s3" {
  statement {
    actions = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject", "s3:ListBucket"]
    resources = [
      "arn:aws:s3:::acme-loki-chunks", "arn:aws:s3:::acme-loki-chunks/*",
      "arn:aws:s3:::acme-loki-ruler",  "arn:aws:s3:::acme-loki-ruler/*",
      "arn:aws:s3:::acme-loki-admin",  "arn:aws:s3:::acme-loki-admin/*",
    ]
  }
}

resource "aws_iam_policy" "loki_s3" { name = "${var.cluster_name}-loki-s3" policy = data.aws_iam_policy_document.loki_s3.json }

module "irsa_loki" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.46"
  role_name = "${var.cluster_name}-loki-s3"
  role_policy_arns = { loki_s3 = aws_iam_policy.loki_s3.arn }
  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["observability:loki"]
    }
  }
}

# -----------------------------------------------------------------------------
# Outputs
# -----------------------------------------------------------------------------
output "cluster_name"            { value = module.eks.cluster_name }
output "cluster_endpoint"        { value = module.eks.cluster_endpoint }
output "cluster_oidc_issuer_url" { value = module.eks.cluster_oidc_issuer_url }
output "karpenter_node_iam_role" { value = module.karpenter.node_iam_role_arn }
output "loki_irsa_role_arn"      { value = module.irsa_loki.iam_role_arn }

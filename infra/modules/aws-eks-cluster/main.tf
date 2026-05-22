# aws-eks-cluster — production-grade EKS module.
# Wraps the upstream terraform-aws-modules/eks/aws module with ACME-opinionated
# hardening: private endpoint, KMS-encrypted secrets, control-plane audit logging,
# no aws-auth ConfigMap (API auth mode), Bottlerocket system NG, Cilium-friendly
# (vpc-cni intentionally omitted), Karpenter via Pod Identity, IRSA OIDC ready.
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.70" }
  }
}

variable "cluster_name"       { type = string }
variable "kubernetes_version" { type = string  default = "1.31" }
variable "vpc_id"             { type = string }
variable "subnet_ids"         { type = list(string) }
variable "tags"               { type = map(string)  default = {} }

resource "aws_kms_key" "eks" {
  description             = "EKS secrets encryption — ${var.cluster_name}"
  deletion_window_in_days = 30
  enable_key_rotation     = true
  tags                    = var.tags
}

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.24"

  cluster_name    = var.cluster_name
  cluster_version = var.kubernetes_version
  vpc_id          = var.vpc_id
  subnet_ids      = var.subnet_ids

  # Hardening defaults
  cluster_endpoint_public_access  = false
  cluster_endpoint_private_access = true
  authentication_mode             = "API"
  enable_cluster_creator_admin_permissions = true

  cluster_enabled_log_types = ["api", "audit", "authenticator", "controllerManager", "scheduler"]
  cluster_encryption_config = { provider_key_arn = aws_kms_key.eks.arn, resources = ["secrets"] }

  # Add-ons — kube-proxy in IPVS-off mode (Cilium takes over)
  cluster_addons = {
    coredns                = { most_recent = true }
    kube-proxy             = { most_recent = true, configuration_values = jsonencode({ mode = "none" }) }
    eks-pod-identity-agent = { most_recent = true }
    aws-ebs-csi-driver     = { most_recent = true }
    snapshot-controller    = { most_recent = true }
  }

  # System node group — Bottlerocket, tainted, hosts critical add-ons until Karpenter takes over
  eks_managed_node_groups = {
    system = {
      ami_type       = "BOTTLEROCKET_x86_64"
      instance_types = ["m6i.large"]
      min_size = 3
      max_size = 6
      desired_size = 3
      labels = { workload = "system" }
      taints = [{ key = "CriticalAddonsOnly", value = "true", effect = "NO_SCHEDULE" }]
      iam_role_additional_policies = { ssm = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore" }
      block_device_mappings = {
        xvda = { device_name = "/dev/xvda", ebs = { volume_type = "gp3", volume_size = 100, encrypted = true, kms_key_id = aws_kms_key.eks.arn, delete_on_termination = true } }
      }
    }
  }

  node_security_group_tags = { "karpenter.sh/discovery" = var.cluster_name }
  tags = var.tags
}

# Karpenter (Pod Identity association created)
module "karpenter" {
  source  = "terraform-aws-modules/eks/aws//modules/karpenter"
  version = "~> 20.24"

  cluster_name              = module.eks.cluster_name
  enable_pod_identity       = true
  create_pod_identity_association = true
  node_iam_role_additional_policies = { ssm = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore" }
  tags = var.tags
}

output "cluster_name"           { value = module.eks.cluster_name }
output "cluster_endpoint"       { value = module.eks.cluster_endpoint }
output "oidc_provider_arn"      { value = module.eks.oidc_provider_arn }
output "oidc_provider_url"      { value = module.eks.cluster_oidc_issuer_url }
output "kms_key_arn"            { value = aws_kms_key.eks.arn }
output "karpenter_node_role"    { value = module.karpenter.node_iam_role_arn }
output "karpenter_queue_name"   { value = module.karpenter.queue_name }

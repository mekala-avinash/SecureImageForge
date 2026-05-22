# argocd-bootstrap — installs Argo CD HA into a fresh cluster + bootstraps the root App.
# Uses helm provider; the cluster must already exist and kube provider configured.
terraform {
  required_version = ">= 1.5"
  required_providers {
    helm       = { source = "hashicorp/helm",       version = "~> 2.15" }
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.32" }
    kubectl    = { source = "alekc/kubectl",        version = "~> 2.0" }
  }
}

variable "argocd_chart_version" { type = string default = "7.7.0" }
variable "gitops_repo_url"      { type = string }    # e.g. https://github.com/acme/gitops
variable "gitops_repo_branch"   { type = string default = "main" }
variable "root_app_path"        { type = string default = "clusters/mgmt-use1" }
variable "namespace"            { type = string default = "argocd" }

resource "kubernetes_namespace" "argocd" {
  metadata {
    name = var.namespace
    labels = {
      "pod-security.kubernetes.io/enforce" = "restricted"
      "pod-security.kubernetes.io/audit"   = "restricted"
      "pod-security.kubernetes.io/warn"    = "restricted"
    }
  }
}

resource "helm_release" "argocd" {
  name             = "argocd"
  repository       = "https://argoproj.github.io/argo-helm"
  chart            = "argo-cd"
  version          = var.argocd_chart_version
  namespace        = kubernetes_namespace.argocd.metadata[0].name
  create_namespace = false
  timeout          = 600

  values = [yamlencode({
    global = { domain = "argocd.acme.io" }
    configs = {
      params = { "server.insecure" = "true" } # mTLS at the mesh edge instead
      cm = {
        "url" = "https://argocd.acme.io"
        "exec.enabled" = "false"  # no kubectl exec from UI
        # OIDC via Keycloak
        "oidc.config" = yamlencode({
          name = "Keycloak"
          issuer = "https://id.acme.io/realms/acme"
          clientID = "argocd"
          requestedScopes = ["openid","profile","email","groups"]
        })
      }
      rbac = {
        "policy.default" = "role:readonly"
        "policy.csv" = <<-EOT
          g, acme:platform-admins, role:admin
          g, acme:sre,             role:admin
          g, acme:everyone,        role:readonly
        EOT
      }
    }
    controller = {
      replicas = 2
      resources = { requests = { cpu = "500m", memory = "1Gi" }, limits = { cpu = "2", memory = "4Gi" } }
    }
    server = {
      replicas = 2
      resources = { requests = { cpu = "100m", memory = "256Mi" }, limits = { cpu = "1", memory = "1Gi" } }
    }
    repoServer = {
      replicas = 2
      resources = { requests = { cpu = "200m", memory = "512Mi" }, limits = { cpu = "1", memory = "2Gi" } }
    }
    applicationSet = { replicas = 2 }
    notifications  = { enabled = true }
  })]
}

# Root App-of-Apps — single Application that points at the GitOps repo;
# Argo CD discovers everything else from there.
resource "kubectl_manifest" "root_app" {
  depends_on = [helm_release.argocd]
  yaml_body = yamlencode({
    apiVersion = "argoproj.io/v1alpha1"
    kind       = "Application"
    metadata = {
      name      = "root"
      namespace = var.namespace
      annotations = { "argocd.argoproj.io/sync-wave" = "-100" }
      finalizers = ["resources-finalizer.argocd.argoproj.io"]
    }
    spec = {
      project = "default"
      source = {
        repoURL        = var.gitops_repo_url
        targetRevision = var.gitops_repo_branch
        path           = var.root_app_path
        directory      = { recurse = true }
      }
      destination = { server = "https://kubernetes.default.svc", namespace = var.namespace }
      syncPolicy = {
        automated   = { selfHeal = true, prune = true }
        syncOptions = ["CreateNamespace=true", "ServerSideApply=true"]
        retry       = { limit = 5, backoff = { duration = "30s", factor = 2, maxDuration = "5m" } }
      }
    }
  })
}

output "argocd_namespace" { value = var.namespace }
output "argocd_url"       { value = "https://argocd.acme.io" }

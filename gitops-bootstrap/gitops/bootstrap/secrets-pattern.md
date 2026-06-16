# Secrets integration on the paved road

All secrets flow through **HashiCorp Vault** → **Secrets-Store CSI Driver** →
**Kubernetes Secret mirror** (optional, for `envFrom`).

```
┌──────────────┐  K8s ServiceAccount  ┌──────────────────┐
│   Vault      │ ◄────── JWT/OIDC ───── │ secrets-store-csi│
│ kv/secret/…  │                       │  (DaemonSet)     │
└──────┬───────┘                       └────────┬─────────┘
       │                                        │
       │ SecretProviderClass references         │
       │ vault paths + objectName aliases       │
       │                                        ▼
       │                          Pod mount: /etc/secrets/<objectName>
       │                          optional: K8s Secret <service>-env (envFrom)
```

## Conventions

- **Vault paths** mirror the service slug: `kv/data/<service>/<purpose>` (e.g. `kv/data/orders/db`).
- **Service identity**: each service has a Vault role bound to its Kubernetes ServiceAccount via the Kubernetes auth method.
- **Rotation**: Vault leases are short (1h for DB creds). The CSI driver re-mounts on rotation; apps must re-read the file.
- **Forbidden**: hand-rolled `kind: Secret` with hard-coded data; checked-in `.env` files; Helm `--set` of plaintext secrets.

## Adding a new secret

1. Operator writes to Vault: `vault kv put kv/<service>/<purpose> key=...`.
2. Developer edits the service's `apps/<service>/base/secret-provider-class.yaml`:
   ```yaml
   - objectName: my-new-secret
     secretPath: kv/data/<service>/<purpose>
     secretKey:  key
   ```
3. PR merge → ArgoCD applies the SecretProviderClass → CSI driver materialises `/etc/secrets/my-new-secret` on next Pod start.

## Emergency rotation

```bash
vault kv put kv/<service>/<purpose> key=$(openssl rand -hex 32)
kubectl -n <service>-<env> rollout restart deploy/<service>
```

The `acme.workload` library template auto-mounts the CSI volume when `.Values.secrets.enabled=true`.

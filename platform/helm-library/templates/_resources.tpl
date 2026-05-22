{{/*
acme.service          — Service + ServiceAccount
acme.hpa              — HorizontalPodAutoscaler
acme.pdb              — PodDisruptionBudget
acme.networkPolicy    — Cilium-flavored NetworkPolicy (default-deny + Istio-gateway ingress + kube-dns egress)
acme.istioAuthz       — Istio AuthorizationPolicy + PeerAuthentication (STRICT mTLS)
acme.serviceMonitor   — Prometheus Operator ServiceMonitor
acme.slo              — Sloth PrometheusServiceLevel (multi-window burn-rate alerts)
*/}}

{{- define "acme.service" -}}
{{- if .Values.serviceAccount.create -}}
apiVersion: v1
kind: ServiceAccount
metadata:
  name: {{ include "acme.serviceAccountName" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
  {{- with .Values.serviceAccount.annotations }}
  annotations: {{- toYaml . | nindent 4 }}
  {{- end }}
automountServiceAccountToken: false
---
{{- end -}}
{{- if .Values.service.enabled -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  selector: {{- include "acme.selectorLabels" . | nindent 4 }}
  ports:
    - name: http
      port: {{ .Values.service.port }}
      targetPort: {{ .Values.service.targetPort }}
      protocol: {{ .Values.service.protocol }}
      appProtocol: {{ .Values.service.appProtocol }}
{{- end -}}
{{- end -}}


{{- define "acme.hpa" -}}
{{- if and .Values.hpa.enabled (eq .Values.kind "deployment") -}}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "acme.fullname" . }}
  minReplicas: {{ .Values.hpa.minReplicas }}
  maxReplicas: {{ .Values.hpa.maxReplicas }}
  metrics:
    - type: Resource
      resource:
        name: cpu
        target: { type: Utilization, averageUtilization: {{ .Values.hpa.targetCPUUtilizationPercentage }} }
    {{- range .Values.hpa.customMetrics }}
    - {{ toYaml . | nindent 6 | trim }}
    {{- end }}
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies: [{ type: Percent, value: 25, periodSeconds: 60 }]
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - { type: Percent, value: 100, periodSeconds: 30 }
        - { type: Pods,    value: 8,   periodSeconds: 30 }
      selectPolicy: Max
{{- end -}}
{{- end -}}


{{- define "acme.pdb" -}}
{{- if .Values.pdb.enabled -}}
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  minAvailable: {{ .Values.pdb.minAvailable }}
  selector:
    matchLabels: {{- include "acme.selectorLabels" . | nindent 6 }}
{{- end -}}
{{- end -}}


{{- define "acme.networkPolicy" -}}
{{- if .Values.networkPolicy.enabled -}}
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  endpointSelector:
    matchLabels: {{- include "acme.selectorLabels" . | nindent 6 }}
  ingress:
    # Allow from Istio ingress gateway
    - fromEndpoints:
        - matchLabels:
            "k8s:io.kubernetes.pod.namespace": istio-system
            "app": istio-ingressgateway
      toPorts:
        - ports: [{ port: {{ .Values.service.targetPort | quote }}, protocol: TCP }]
    {{- with .Values.networkPolicy.extraIngress }}
    {{- toYaml . | nindent 4 }}
    {{- end }}
  egress:
    # kube-dns
    - toEndpoints:
        - matchLabels:
            "k8s:io.kubernetes.pod.namespace": kube-system
            "k8s:k8s-app": kube-dns
      toPorts:
        - ports: [{ port: "53", protocol: UDP }, { port: "53", protocol: TCP }]
    # OTel collector
    - toEndpoints:
        - matchLabels:
            "k8s:io.kubernetes.pod.namespace": observability
            "app.kubernetes.io/name": opentelemetry-collector
      toPorts:
        - ports: [{ port: "4317", protocol: TCP }]
    {{- with .Values.networkPolicy.extraEgress }}
    {{- toYaml . | nindent 4 }}
    {{- end }}
{{- end -}}
{{- end -}}


{{- define "acme.istioAuthz" -}}
{{- if .Values.istio.enabled -}}
apiVersion: security.istio.io/v1
kind: PeerAuthentication
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  selector:
    matchLabels: {{- include "acme.selectorLabels" . | nindent 6 }}
  mtls: { mode: {{ .Values.istio.mtls }} }
---
{{- if .Values.istio.authorization.enabled }}
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  selector:
    matchLabels: {{- include "acme.selectorLabels" . | nindent 6 }}
  action: ALLOW
  rules:
    {{- if .Values.istio.authorization.principals }}
    - from:
        - source:
            principals: {{- toYaml .Values.istio.authorization.principals | nindent 14 }}
    {{- else }}
    # Default: allow from istio-ingressgateway only.
    - from:
        - source:
            principals: ["cluster.local/ns/istio-system/sa/istio-ingressgateway"]
    {{- end }}
{{- end -}}
{{- end -}}
{{- end -}}


{{- define "acme.serviceMonitor" -}}
{{- if .Values.serviceMonitor.enabled -}}
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: {{ include "acme.fullname" . }}
  labels:
    {{- include "acme.labels" . | nindent 4 }}
    release: kube-prometheus-stack
spec:
  selector:
    matchLabels: {{- include "acme.selectorLabels" . | nindent 6 }}
  endpoints:
    - port: {{ .Values.serviceMonitor.port }}
      path: {{ .Values.serviceMonitor.path }}
      interval: {{ .Values.serviceMonitor.interval }}
      scrapeTimeout: {{ .Values.serviceMonitor.scrapeTimeout }}
{{- end -}}
{{- end -}}


{{- define "acme.slo" -}}
{{- if .Values.slos.enabled -}}
apiVersion: sloth.slok.dev/v1
kind: PrometheusServiceLevel
metadata:
  name: {{ include "acme.fullname" . }}
  labels:
    {{- include "acme.labels" . | nindent 4 }}
    slo: "true"
spec:
  service: {{ .Values.name }}
  labels:
    team: {{ .Values.team }}
  slos:
    {{- range .Values.slos.objectives }}
    - name: {{ .name }}
      objective: {{ .objective }}
      sli:
        events:
          error_query: {{ tpl .sli.errorQuery $ | quote }}
          total_query: {{ tpl .sli.totalQuery $ | quote }}
      alerting:
        name: {{ printf "%s-%s" $.Values.name .name }}
        page_alert:   { labels: { severity: page,   team: {{ $.Values.team | quote }} }, annotations: { runbook_url: "https://backstage.acme.io/docs/default/component/{{ $.Values.name }}/runbooks/{{ .name }}" } }
        ticket_alert: { labels: { severity: ticket, team: {{ $.Values.team | quote }} } }
    {{- end }}
{{- end -}}
{{- end -}}

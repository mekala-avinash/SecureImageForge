{{/*
acme.workload — emits a Deployment OR a Rollout depending on values.kind.

Usage in a service chart:

  {{- include "acme.workload" . }}

The template enforces:
  - non-root + read-only FS + dropped caps
  - resources requests + memory limit
  - startup/liveness/readiness probes
  - topology spread
  - OTel resource attributes
*/}}
{{- define "acme.workload" -}}
{{- if eq .Values.kind "rollout" -}}
apiVersion: argoproj.io/v1alpha1
kind: Rollout
{{- else -}}
apiVersion: apps/v1
kind: Deployment
{{- end }}
metadata:
  name: {{ include "acme.fullname" . }}
  labels: {{- include "acme.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  revisionHistoryLimit: {{ .Values.revisionHistoryLimit }}
  selector:
    matchLabels: {{- include "acme.selectorLabels" . | nindent 6 }}
  {{- if eq .Values.kind "rollout" }}
  strategy: {{- toYaml .Values.rollout.strategy | nindent 4 }}
  {{- else }}
  strategy:
    type: RollingUpdate
    rollingUpdate: { maxSurge: 25%, maxUnavailable: 0 }
  {{- end }}
  template:
    metadata:
      labels:
        {{- include "acme.selectorLabels" . | nindent 8 }}
        {{- with .Values.podLabels }}{{- toYaml . | nindent 8 }}{{- end }}
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: {{ .Values.service.targetPort | quote }}
        prometheus.io/path: "/metrics"
        {{- if .Values.otel.enabled }}
        instrumentation.opentelemetry.io/inject-sdk: {{ if eq .Values.otel.sdk "auto" }}"true"{{ else }}"false"{{ end }}
        {{- end }}
        {{- with .Values.podAnnotations }}{{- toYaml . | nindent 8 }}{{- end }}
    spec:
      automountServiceAccountToken: false
      serviceAccountName: {{ include "acme.serviceAccountName" . }}
      securityContext: {{- toYaml .Values.podSecurityContext | nindent 8 }}
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets: {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.topologySpread }}
      topologySpreadConstraints:
        {{- range . }}
        - {{ toYaml . | nindent 10 | trim }}
          labelSelector:
            matchLabels: {{- include "acme.selectorLabels" $ | nindent 14 }}
        {{- end }}
      {{- end }}
      containers:
        - name: app
          image: {{ include "acme.image" . | quote }}
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          securityContext: {{- toYaml .Values.containerSecurityContext | nindent 12 }}
          ports:
            - name: http
              containerPort: {{ .Values.service.targetPort }}
              protocol: {{ .Values.service.protocol }}
          env:
            - name: PORT
              value: {{ .Values.service.targetPort | quote }}
            {{- if .Values.otel.enabled }}
            - name: OTEL_SERVICE_NAME
              value: {{ default .Values.name .Values.otel.serviceName | quote }}
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: "http://otel-collector.observability:4317"
            - name: OTEL_RESOURCE_ATTRIBUTES
              value: "service.name={{ default .Values.name .Values.otel.serviceName }},service.namespace={{ .Release.Namespace }},team={{ .Values.team }}{{- range $k, $v := .Values.otel.resourceAttributes }}{{- if $v }},{{$k}}={{$v}}{{- end }}{{- end }}"
            {{- end }}
            {{- with .Values.env }}{{- toYaml . | nindent 12 }}{{- end }}
          {{- with .Values.envFrom }}
          envFrom: {{- toYaml . | nindent 12 }}
          {{- end }}
          resources: {{- toYaml .Values.resources | nindent 12 }}
          startupProbe:   {{- toYaml .Values.probes.startup   | nindent 12 }}
          livenessProbe:  {{- toYaml .Values.probes.liveness  | nindent 12 }}
          readinessProbe: {{- toYaml .Values.probes.readiness | nindent 12 }}
          lifecycle:
            preStop: { exec: { command: ["/bin/sh", "-c", "sleep 10"] } }
          volumeMounts:
            - { name: tmp, mountPath: /tmp }
            {{- if .Values.secrets.enabled }}
            - name: secrets-store
              mountPath: {{ .Values.secrets.mountPath }}
              readOnly: true
            {{- end }}
            {{- with .Values.extraVolumeMounts }}{{- toYaml . | nindent 12 }}{{- end }}
      volumes:
        - name: tmp
          emptyDir: { sizeLimit: 64Mi, medium: Memory }
        {{- if .Values.secrets.enabled }}
        - name: secrets-store
          csi:
            driver: secrets-store.csi.k8s.io
            readOnly: true
            volumeAttributes:
              secretProviderClass: {{ required "secrets.spcName required" .Values.secrets.spcName }}
        {{- end }}
        {{- with .Values.extraVolumes }}{{- toYaml . | nindent 8 }}{{- end }}
{{- end -}}

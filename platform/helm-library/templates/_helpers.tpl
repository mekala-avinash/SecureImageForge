{{/*
acme.fullname — service name (already namespaced via Release.Namespace).
Helpers used by every template.
*/}}
{{- define "acme.name" -}}
{{- required "values.name is required" .Values.name -}}
{{- end -}}

{{- define "acme.fullname" -}}
{{- include "acme.name" . -}}
{{- end -}}

{{/*
Standard labels — applied to every resource.
Includes team for cost allocation + alert routing.
*/}}
{{- define "acme.labels" -}}
app.kubernetes.io/name: {{ include "acme.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/component: service
app.kubernetes.io/part-of: acme-platform
acme.io/team: {{ required "values.team is required" .Values.team | quote }}
{{- with .Values.extraLabels }}
{{ toYaml . }}
{{- end }}
{{- end -}}

{{/*
Selector labels — strict subset that never changes (no version, no chart).
*/}}
{{- define "acme.selectorLabels" -}}
app.kubernetes.io/name: {{ include "acme.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Image — always require a digest in production overlays. Tags only OK in dev.
*/}}
{{- define "acme.image" -}}
{{- $repo := required "values.image.repository is required" .Values.image.repository -}}
{{- if .Values.image.digest -}}
{{ $repo }}@{{ .Values.image.digest }}
{{- else if .Values.image.tag -}}
{{ $repo }}:{{ .Values.image.tag }}
{{- else -}}
{{ fail "values.image.digest (preferred) or values.image.tag is required" }}
{{- end -}}
{{- end -}}

{{/*
serviceAccountName — defaults to service name when create=true.
*/}}
{{- define "acme.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{ include "acme.name" . }}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

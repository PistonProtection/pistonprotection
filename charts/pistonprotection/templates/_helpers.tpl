{{/*
Expand the name of the chart.
*/}}
{{- define "pistonprotection.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "pistonprotection.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "pistonprotection.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "pistonprotection.labels" -}}
helm.sh/chart: {{ include "pistonprotection.chart" . }}
{{ include "pistonprotection.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "pistonprotection.selectorLabels" -}}
app.kubernetes.io/name: {{ include "pistonprotection.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "pistonprotection.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "pistonprotection.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Gateway component labels
*/}}
{{- define "pistonprotection.gateway.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: gateway
{{- end }}

{{- define "pistonprotection.gateway.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: gateway
{{- end }}

{{/*
Worker component labels
*/}}
{{- define "pistonprotection.worker.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: worker
{{- end }}

{{- define "pistonprotection.worker.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: worker
{{- end }}

{{/*
Operator component labels
*/}}
{{- define "pistonprotection.operator.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: operator
{{- end }}

{{- define "pistonprotection.operator.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: operator
{{- end }}

{{/*
Frontend component labels
*/}}
{{- define "pistonprotection.frontend.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: frontend
{{- end }}

{{- define "pistonprotection.frontend.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: frontend
{{- end }}

{{/*
Auth component labels
*/}}
{{- define "pistonprotection.auth.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: auth
{{- end }}

{{- define "pistonprotection.auth.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: auth
{{- end }}

{{/*
Metrics component labels
*/}}
{{- define "pistonprotection.metrics.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: metrics
{{- end }}

{{- define "pistonprotection.metrics.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: metrics
{{- end }}

{{/*
Config Manager component labels
*/}}
{{- define "pistonprotection.configMgr.labels" -}}
{{ include "pistonprotection.labels" . }}
app.kubernetes.io/component: config-mgr
{{- end }}

{{- define "pistonprotection.configMgr.selectorLabels" -}}
{{ include "pistonprotection.selectorLabels" . }}
app.kubernetes.io/component: config-mgr
{{- end }}

{{/*
Database connection URL
*/}}
{{- define "pistonprotection.databaseUrl" -}}
{{- if .Values.postgresql.enabled }}
postgresql://{{ .Values.postgresql.auth.username }}:{{ .Values.postgresql.auth.password }}@{{ include "pistonprotection.fullname" . }}-postgresql:5432/{{ .Values.postgresql.auth.database }}
{{- else }}
postgresql://{{ .Values.postgresql.external.username }}:{{ .Values.postgresql.external.password }}@{{ .Values.postgresql.external.host }}:{{ .Values.postgresql.external.port }}/{{ .Values.postgresql.external.database }}
{{- end }}
{{- end }}

{{/*
Redis connection URL
*/}}
{{- define "pistonprotection.redisUrl" -}}
{{- if .Values.redis.enabled }}
redis://:{{ .Values.redis.auth.password }}@{{ include "pistonprotection.fullname" . }}-redis-master:6379
{{- else }}
redis://:{{ .Values.redis.external.password }}@{{ .Values.redis.external.host }}:{{ .Values.redis.external.port }}
{{- end }}
{{- end }}

{{/*
Image tag
*/}}
{{- define "pistonprotection.imageTag" -}}
{{- .Values.image.tag | default .Chart.AppVersion }}
{{- end }}

{{/*
Loki/Alloy annotations for log collection
Uses Grafana Alloy (promtail replacement) annotation patterns
*/}}
{{- define "pistonprotection.lokiAnnotations" -}}
{{- if .Values.observability.loki.enabled }}
# Grafana Alloy log collection annotations (replaces deprecated promtail.io)
alloy.grafana.com/logs.enabled: "true"
alloy.grafana.com/logs.job: "pistonprotection"
{{- if .Values.observability.loki.tenantId }}
alloy.grafana.com/logs.tenant: {{ .Values.observability.loki.tenantId | quote }}
{{- end }}
{{- range $key, $value := .Values.observability.loki.labels }}
alloy.grafana.com/logs.{{ $key }}: {{ $value | quote }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Common pod annotations including Loki integration
*/}}
{{- define "pistonprotection.podAnnotations" -}}
{{- include "pistonprotection.lokiAnnotations" . }}
{{- end }}

{{/*
Return PostgreSQL hostname
*/}}
{{- define "pistonprotection.postgresql.host" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "pistonprotection.fullname" .) }}
{{- else }}
{{- .Values.postgresql.external.host }}
{{- end }}
{{- end }}

{{/*
Return PostgreSQL port
*/}}
{{- define "pistonprotection.postgresql.port" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "5432" }}
{{- else }}
{{- .Values.postgresql.external.port | toString }}
{{- end }}
{{- end }}

{{/*
Return Redis hostname
*/}}
{{- define "pistonprotection.redis.host" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-redis-master" (include "pistonprotection.fullname" .) }}
{{- else }}
{{- .Values.redis.external.host }}
{{- end }}
{{- end }}

{{/*
Return Redis port
*/}}
{{- define "pistonprotection.redis.port" -}}
{{- if .Values.redis.enabled }}
{{- printf "6379" }}
{{- else }}
{{- .Values.redis.external.port | toString }}
{{- end }}
{{- end }}

{{/*
Return the appropriate API version for HorizontalPodAutoscaler
*/}}
{{- define "pistonprotection.hpa.apiVersion" -}}
{{- if .Capabilities.APIVersions.Has "autoscaling/v2" }}
{{- print "autoscaling/v2" }}
{{- else }}
{{- print "autoscaling/v2beta2" }}
{{- end }}
{{- end }}

{{/*
Return the appropriate API version for PodDisruptionBudget
*/}}
{{- define "pistonprotection.pdb.apiVersion" -}}
{{- if .Capabilities.APIVersions.Has "policy/v1" }}
{{- print "policy/v1" }}
{{- else }}
{{- print "policy/v1beta1" }}
{{- end }}
{{- end }}

{{/*
Return the appropriate API version for NetworkPolicy
*/}}
{{- define "pistonprotection.networkPolicy.apiVersion" -}}
{{- print "networking.k8s.io/v1" }}
{{- end }}

{{/*
Checksum for config and secrets to trigger pod restarts on changes
*/}}
{{- define "pistonprotection.configChecksum" -}}
checksum/config: {{ include (print $.Template.BasePath "/configmaps.yaml") . | sha256sum }}
{{- end }}

{{- define "pistonprotection.secretChecksum" -}}
checksum/secret: {{ include (print $.Template.BasePath "/secrets.yaml") . | sha256sum }}
{{- end }}

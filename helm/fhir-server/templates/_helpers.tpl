{{/*
Expand the name of the chart.
*/}}
{{- define "fhir-server.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "fhir-server.fullname" -}}
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
{{- define "fhir-server.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "fhir-server.labels" -}}
helm.sh/chart: {{ include "fhir-server.chart" . }}
{{ include "fhir-server.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "fhir-server.selectorLabels" -}}
app.kubernetes.io/name: {{ include "fhir-server.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "fhir-server.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "fhir-server.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
PostgreSQL host
*/}}
{{- define "fhir-server.postgresql.host" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "fhir-server.fullname" .) }}
{{- else }}
{{- .Values.externalDatabase.host }}
{{- end }}
{{- end }}

{{/*
PostgreSQL port
*/}}
{{- define "fhir-server.postgresql.port" -}}
{{- if .Values.postgresql.enabled }}
{{- 5432 }}
{{- else }}
{{- .Values.externalDatabase.port }}
{{- end }}
{{- end }}

{{/*
PostgreSQL database name
*/}}
{{- define "fhir-server.postgresql.database" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.database }}
{{- else }}
{{- .Values.externalDatabase.database }}
{{- end }}
{{- end }}

{{/*
PostgreSQL username
*/}}
{{- define "fhir-server.postgresql.username" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.username }}
{{- else }}
{{- .Values.externalDatabase.username }}
{{- end }}
{{- end }}

{{/*
PostgreSQL secret name
*/}}
{{- define "fhir-server.postgresql.secretName" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "fhir-server.fullname" .) }}
{{- else if .Values.externalDatabase.existingSecret }}
{{- .Values.externalDatabase.existingSecret }}
{{- else }}
{{- printf "%s-external-db" (include "fhir-server.fullname" .) }}
{{- end }}
{{- end }}

{{/*
PostgreSQL secret key
*/}}
{{- define "fhir-server.postgresql.secretKey" -}}
{{- if .Values.postgresql.enabled }}
{{- "password" }}
{{- else if .Values.externalDatabase.existingSecret }}
{{- .Values.externalDatabase.existingSecretPasswordKey }}
{{- else }}
{{- "password" }}
{{- end }}
{{- end }}

{{/*
Database URL
*/}}
{{- define "fhir-server.databaseURL" -}}
{{- printf "postgres://%s:$(DATABASE_PASSWORD)@%s:%s/%s" (include "fhir-server.postgresql.username" .) (include "fhir-server.postgresql.host" .) (include "fhir-server.postgresql.port" . | toString) (include "fhir-server.postgresql.database" .) }}
{{- end }}

{{/*
JWT secret name
*/}}
{{- define "fhir-server.jwtSecretName" -}}
{{- if .Values.jwtSecret.existingSecret }}
{{- .Values.jwtSecret.existingSecret }}
{{- else }}
{{- printf "%s-jwt" (include "fhir-server.fullname" .) }}
{{- end }}
{{- end }}

{{/*
gRPC TLS secret name
*/}}
{{- define "fhir-server.grpcTLSSecretName" -}}
{{- if .Values.grpcTLS.existingSecret }}
{{- .Values.grpcTLS.existingSecret }}
{{- else }}
{{- printf "%s-grpc-tls" (include "fhir-server.fullname" .) }}
{{- end }}
{{- end }}

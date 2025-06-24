{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "csmock.name" -}}
{{- printf "%s-csmock" .Release.Name | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "smoketesting.service.name" -}}
{{- printf "%s-smoketesting" .Release.Name | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "smoketesting.service.domainname" -}}
{{- printf "%s-smoketesting.%s.svc.cluster.local" .Release.Name .Release.Namespace | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "csmock.service.domainname" -}}
{{- printf "%s-csmock.%s.svc.cluster.local" .Release.Name .Release.Namespace | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "csmock.service.domainname-ephemeral-generic" -}}
{{- printf "ephemeral-generic.default.%s-csmock.%s.svc.cluster.local" .Release.Name .Release.Namespace | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "client.service.domainname" -}}
{{- printf "%s-client-service.%s.svc.cluster.local" .Release.Name .Release.Namespace | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "coord.service.domainname" -}}
{{- printf "%s-coordination-service.%s.svc.cluster.local" .Release.Name .Release.Namespace | trunc 63 | trimSuffix "-" }}
{{- end }}
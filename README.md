# HelmCharts for Glaciation UC3

This branch is used as a repository for the helm charts of UC3.

## Usage

To use the charts run
```bash
helm repo add glaciation-uc3 https://glaciation-heu.github.io/sap-uc3
helm repo update
```

To install only the coordination service run
```bash
helm install uc3-coord glaciation-uc3/coordination-service
```

To install only the client service run
```bash
helm install uc3-client glaciation-uc3/client-service
```

To install all services, including smoketesting and a cs-mock implementation run
```bash
helm install uc3 glaciation-uc3/secure-collab-computation
```

## Configuration

Coordination service yaml configuraiton
```yaml
namespace: default
service:
  type: ClusterIP
  port: 80
  loadBalancerIP: 172.18.3.129

deployment:
  image: ghcr.io/glaciation-heu/sap-uc3/coordination_service:latest
  pullSecret: ""
  replicas: 1
  port: 80
  resources:
    requests:
      memory: "64Mi"
      cpu: "250m"
    limits: 
      memory: "128Mi"
      cpu: "500m"

config:
  # Service uri for the swagger server.
  swagger_server_uri: ""
  log_level: info
  service_prefix: ""

#DB configuration
postgresql:
  auth:
    username: coord
    # TODO change to a secure password
    password: coord
    database: coorddb
  primary:
    podSecurityContext:
        enabled: false
        fsGroup: ""
    containerSecurityContext:
        enabled: false
        runAsUser: "auto"

readReplicas:
    podSecurityContext:
        enabled: false
        fsGroup: ""
    containerSecurityContext:
        enabled: false
        runAsUser: "auto"

volumePermissions:
    enabled: false
    securityContext:
        runAsUser: "auto"
```

Client service yaml configuration:
```yaml
service:
  type: ClusterIP
  port: 80
  loadBalancerIP: 172.18.3.128

deployment:
  image: ghcr.io/glaciation-heu/sap-uc3/client_service:latest
  pullSecret: ""
  replicas: 1
  resources:
    requests:
      memory: "64Mi"
      cpu: "250m"
    limits: 
      memory: "128Mi"
      cpu: "500m"

config:
  # Will enable a swagger API-Service on this URI
  swagger_server_uri: ""
  coordinator_uri: "http://coordination-service.svc.cluster.local"
  log_level: info
  service_prefix: ""
```

Secure collaborative computation yaml configuration:
```yaml
client-service:
  enabled: true
  config:
    coordinator_uri: "http://uc3-coordination-service.default.svc.cluster.local/coord"
    swagger_server_uri: "http://localhost/client"
    log_level: debug
    service_prefix: /client
  deployment:
    pullSecret: dockerconfigjson-github-com

csmock: 
  enabled: true
  config:
    swagger_server_uri: "http://localhost/csmock"
    log_level: debug
    service_prefix: /csmock
  deployment:
    pullSecret: dockerconfigjson-github-com

coordination-service:
  config:
    swagger_server_uri: "http://localhost/coord"
    log_level: debug
    service_prefix: /coord
  deployment:
    pullSecret: dockerconfigjson-github-com
  postgresql:
    auth:
      username: coord
      # TODO change to a secure password
      password: coord
      database: coorddb

ingress:
  namespace: ingress-nginx
  enabled: true
  # host: localhost
```

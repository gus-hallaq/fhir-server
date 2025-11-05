# FHIR Server Helm Chart

This Helm chart deploys the FHIR Server with both HTTP REST API and gRPC services on a Kubernetes cluster.

## Features

- ✅ **Multiple Replicas**: Deploy 3+ replicas for high availability
- ✅ **Horizontal Pod Autoscaling**: Automatic scaling based on CPU/Memory
- ✅ **Pod Disruption Budget**: Ensure availability during updates
- ✅ **HTTP & gRPC Services**: Dual protocol support
- ✅ **Ingress Support**: External access with TLS termination
- ✅ **PostgreSQL Integration**: Bundled or external database
- ✅ **Security**: Pod security contexts, read-only root filesystem
- ✅ **Observability**: Health checks, readiness/liveness probes
- ✅ **Anti-Affinity**: Distribute pods across nodes

## Prerequisites

- Kubernetes 1.20+
- Helm 3.0+
- PV provisioner support in the underlying infrastructure (if using persistent storage)
- Optional: Ingress controller (nginx recommended)
- Optional: cert-manager for TLS certificates

## Installation

### Quick Start

```bash
# Add the repository (if published)
helm repo add fhir-server https://charts.example.com
helm repo update

# Install with default values
helm install my-fhir-server fhir-server/fhir-server

# Or install from local chart
helm install my-fhir-server ./helm/fhir-server
```

### Custom Values

```bash
# Install with custom values
helm install my-fhir-server ./helm/fhir-server \
  --set replicaCount=5 \
  --set postgresql.auth.password=secure-password \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=fhir.example.com

# Install with values file
helm install my-fhir-server ./helm/fhir-server -f my-values.yaml
```

## Configuration

### Key Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas (ignored if autoscaling enabled) | `3` |
| `image.repository` | Image repository | `fhir-server` |
| `image.tag` | Image tag | `latest` |
| `autoscaling.enabled` | Enable HorizontalPodAutoscaler | `true` |
| `autoscaling.minReplicas` | Minimum replicas | `3` |
| `autoscaling.maxReplicas` | Maximum replicas | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | Target CPU % | `70` |
| `autoscaling.targetMemoryUtilizationPercentage` | Target Memory % | `80` |

### Service Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `httpService.type` | Kubernetes service type for HTTP | `ClusterIP` |
| `httpService.port` | HTTP service port | `8080` |
| `grpcService.type` | Kubernetes service type for gRPC | `ClusterIP` |
| `grpcService.port` | gRPC service port | `50051` |

### Ingress Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable HTTP ingress | `false` |
| `ingress.className` | Ingress class name | `nginx` |
| `ingress.hosts[0].host` | HTTP hostname | `fhir.example.com` |
| `grpcIngress.enabled` | Enable gRPC ingress | `false` |
| `grpcIngress.hosts[0].host` | gRPC hostname | `grpc.fhir.example.com` |

### Database Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `postgresql.enabled` | Deploy PostgreSQL with chart | `true` |
| `postgresql.auth.username` | PostgreSQL username | `fhir` |
| `postgresql.auth.password` | PostgreSQL password | `fhir_password` |
| `postgresql.auth.database` | PostgreSQL database name | `fhir` |
| `externalDatabase.host` | External DB host (if postgresql.enabled=false) | `postgres.example.com` |

### Resources

| Parameter | Description | Default |
|-----------|-------------|---------|
| `resources.requests.cpu` | CPU request | `250m` |
| `resources.requests.memory` | Memory request | `256Mi` |
| `resources.limits.cpu` | CPU limit | `1000m` |
| `resources.limits.memory` | Memory limit | `512Mi` |

## Example Configurations

### Production Deployment with External Database

```yaml
# production-values.yaml
replicaCount: 5

image:
  repository: your-registry.com/fhir-server
  tag: "1.0.0"
  pullPolicy: IfNotPresent

autoscaling:
  enabled: true
  minReplicas: 5
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70

postgresql:
  enabled: false

externalDatabase:
  host: postgres.prod.example.com
  port: 5432
  username: fhir_prod
  existingSecret: fhir-db-secret
  existingSecretPasswordKey: password
  database: fhir_production

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: fhir.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: fhir-server-tls
      hosts:
        - fhir.example.com

grpcIngress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/backend-protocol: "GRPC"
  hosts:
    - host: grpc.fhir.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: fhir-grpc-tls
      hosts:
        - grpc.fhir.example.com

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

jwtSecret:
  existingSecret: fhir-jwt-secret
  key: jwt-secret

config:
  grpc:
    tlsEnabled: true

grpcTLS:
  existingSecret: fhir-grpc-tls-certs
```

### Development Deployment

```yaml
# dev-values.yaml
replicaCount: 1

autoscaling:
  enabled: false

postgresql:
  enabled: true
  auth:
    password: dev-password

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: fhir.dev.local
      paths:
        - path: /
          pathType: Prefix

config:
  logging:
    level: debug
```

## Deployment Commands

### Install

```bash
# Development
helm install fhir-dev ./helm/fhir-server -f dev-values.yaml

# Production
helm install fhir-prod ./helm/fhir-server -f production-values.yaml \
  --namespace fhir-production --create-namespace
```

### Upgrade

```bash
# Upgrade with new values
helm upgrade fhir-prod ./helm/fhir-server -f production-values.yaml

# Upgrade with specific values
helm upgrade fhir-prod ./helm/fhir-server \
  --set image.tag=1.1.0 \
  --set replicaCount=10
```

### Rollback

```bash
# List revisions
helm history fhir-prod

# Rollback to previous version
helm rollback fhir-prod

# Rollback to specific revision
helm rollback fhir-prod 3
```

### Uninstall

```bash
helm uninstall fhir-prod
```

## Accessing the Application

### Port Forward (Development)

```bash
# HTTP API
kubectl port-forward svc/fhir-dev-http 8080:8080

# gRPC
kubectl port-forward svc/fhir-dev-grpc 50051:50051
```

### Via Ingress (Production)

```bash
# HTTP API
curl https://fhir.example.com/health

# gRPC (with grpcurl)
grpcurl grpc.fhir.example.com:443 list
```

## Monitoring

### View Logs

```bash
# All pods
kubectl logs -l app.kubernetes.io/name=fhir-server -f

# Specific pod
kubectl logs fhir-server-xxxxx-yyyyy -f
```

### Check HPA Status

```bash
kubectl get hpa fhir-server
kubectl describe hpa fhir-server
```

### Check Pod Distribution

```bash
kubectl get pods -l app.kubernetes.io/name=fhir-server -o wide
```

## Troubleshooting

### Pods Not Starting

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=fhir-server

# Describe pod
kubectl describe pod fhir-server-xxxxx-yyyyy

# Check events
kubectl get events --sort-by='.lastTimestamp'
```

### Database Connection Issues

```bash
# Check database secret
kubectl get secret fhir-server-postgresql -o yaml

# Test database connection from pod
kubectl exec -it fhir-server-xxxxx-yyyyy -- sh
# Inside pod:
# psql -h fhir-server-postgresql -U fhir -d fhir
```

### Ingress Not Working

```bash
# Check ingress
kubectl get ingress
kubectl describe ingress fhir-server-http

# Check ingress controller logs
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller
```

## Security Considerations

1. **Secrets Management**: Use external secret management (e.g., HashiCorp Vault, AWS Secrets Manager)
2. **Network Policies**: Implement NetworkPolicy to restrict traffic
3. **Pod Security**: Chart includes security contexts, but review based on your requirements
4. **TLS**: Always enable TLS for production (both HTTP Ingress and gRPC)
5. **Database**: Use managed database services in production
6. **RBAC**: Configure appropriate ServiceAccount permissions

## Maintenance

### Scaling

```bash
# Manual scaling (if autoscaling disabled)
kubectl scale deployment fhir-server --replicas=10

# Update HPA
helm upgrade fhir-prod ./helm/fhir-server \
  --set autoscaling.minReplicas=5 \
  --set autoscaling.maxReplicas=20
```

### Database Backups

```bash
# If using bundled PostgreSQL
kubectl exec -it fhir-server-postgresql-0 -- sh
pg_dump -U fhir fhir > backup.sql
```

## Advanced Configuration

### Custom Affinity Rules

```yaml
affinity:
  podAntiAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchExpressions:
            - key: app.kubernetes.io/name
              operator: In
              values:
                - fhir-server
        topologyKey: kubernetes.io/hostname
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
        - matchExpressions:
            - key: node-type
              operator: In
              values:
                - compute
```

### Custom Volume Mounts

```yaml
volumeMounts:
  - name: custom-config
    mountPath: /etc/config
    readOnly: true

volumes:
  - name: custom-config
    configMap:
      name: my-custom-config
```

## Contributing

Contributions are welcome! Please submit pull requests or open issues.

## License

[Add your license here]

## Support

For support, please contact [your-email@example.com] or open an issue.

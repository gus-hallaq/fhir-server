# Kubernetes Deployment Guide

This guide walks you through deploying the FHIR Server on Kubernetes using Helm.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Development Deployment](#development-deployment)
- [Production Deployment](#production-deployment)
- [Configuration](#configuration)
- [Scaling](#scaling)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Tools

```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/

# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Verify installation
kubectl version --client
helm version
```

### Kubernetes Cluster

You need access to a Kubernetes cluster. Options include:

- **Local Development**: minikube, kind, k3s, Docker Desktop
- **Cloud Providers**: EKS (AWS), GKE (Google Cloud), AKS (Azure)
- **On-Premise**: kubeadm, Rancher, OpenShift

```bash
# Verify cluster access
kubectl cluster-info
kubectl get nodes
```

### Optional Components

```bash
# Install NGINX Ingress Controller
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml

# Install cert-manager (for TLS certificates)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Verify cert-manager
kubectl get pods --namespace cert-manager
```

## Quick Start

### 1. Build Docker Image

```bash
# Build the image
docker build -t fhir-server:latest .

# If using kind, load image into cluster
kind load docker-image fhir-server:latest

# If using minikube, use minikube's docker daemon
eval $(minikube docker-env)
docker build -t fhir-server:latest .
```

### 2. Deploy with Helm

```bash
# Install the chart
helm install fhir-server ./helm/fhir-server

# Wait for deployment
kubectl wait --for=condition=available --timeout=300s deployment/fhir-server

# Check status
kubectl get pods -l app.kubernetes.io/name=fhir-server
```

### 3. Access the Application

```bash
# Port forward HTTP
kubectl port-forward svc/fhir-server-http 8080:8080

# Port forward gRPC
kubectl port-forward svc/fhir-server-grpc 50051:50051

# Test HTTP API
curl http://localhost:8080/health

# Test gRPC (requires grpcurl)
grpcurl -plaintext localhost:50051 list
```

## Development Deployment

### Local Kubernetes Setup

#### Using minikube

```bash
# Start minikube
minikube start --cpus=4 --memory=8192

# Enable ingress
minikube addons enable ingress

# Deploy
helm install fhir-dev ./helm/fhir-server -f helm/fhir-server/values-development.yaml

# Access via minikube tunnel (in separate terminal)
minikube tunnel

# Add to /etc/hosts
echo "$(minikube ip) fhir.dev.local grpc.fhir.dev.local" | sudo tee -a /etc/hosts

# Access the application
curl http://fhir.dev.local/health
grpcurl -plaintext grpc.fhir.dev.local:80 list
```

#### Using kind

```bash
# Create cluster with ingress support
cat <<EOF | kind create cluster --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
EOF

# Install NGINX Ingress
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

# Wait for ingress
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=90s

# Load image and deploy
kind load docker-image fhir-server:dev
helm install fhir-dev ./helm/fhir-server -f helm/fhir-server/values-development.yaml

# Add to /etc/hosts
echo "127.0.0.1 fhir.dev.local grpc.fhir.dev.local" | sudo tee -a /etc/hosts

# Access the application
curl http://fhir.dev.local/health
```

### Development Workflow

```bash
# Make code changes...

# Rebuild image
docker build -t fhir-server:dev .

# Load into cluster
kind load docker-image fhir-server:dev  # or minikube image load fhir-server:dev

# Restart pods to use new image
kubectl rollout restart deployment/fhir-dev

# Watch rollout
kubectl rollout status deployment/fhir-dev

# View logs
kubectl logs -l app.kubernetes.io/name=fhir-server -f
```

## Production Deployment

### 1. Prepare Environment

```bash
# Create namespace
kubectl create namespace fhir-production

# Create secrets
# Database password
kubectl create secret generic fhir-db-secret \
  --from-literal=password='your-secure-password' \
  --namespace fhir-production

# JWT secret
kubectl create secret generic fhir-jwt-secret \
  --from-literal=jwt-secret='your-jwt-secret-key' \
  --namespace fhir-production

# gRPC TLS certificates (if using TLS)
kubectl create secret tls fhir-grpc-tls-certs \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  --namespace fhir-production
```

### 2. Configure cert-manager (Optional)

```bash
# Create ClusterIssuer for Let's Encrypt
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

### 3. Deploy Application

```bash
# Install with production values
helm install fhir-prod ./helm/fhir-server \
  --namespace fhir-production \
  --values helm/fhir-server/values-production.yaml \
  --set image.tag=1.0.0 \
  --set externalDatabase.password='' \
  --set jwtSecret.value=''

# Watch deployment
kubectl get pods -n fhir-production -w

# Check rollout status
kubectl rollout status deployment/fhir-prod -n fhir-production
```

### 4. Verify Deployment

```bash
# Check all resources
kubectl get all -n fhir-production

# Check ingress
kubectl get ingress -n fhir-production

# Check certificates (if using cert-manager)
kubectl get certificate -n fhir-production

# Test endpoints
curl https://fhir.example.com/health
grpcurl grpc.fhir.example.com:443 list
```

## Configuration

### Database Configuration

#### Using External PostgreSQL

```yaml
postgresql:
  enabled: false

externalDatabase:
  host: postgres.example.com
  port: 5432
  username: fhir
  database: fhir
  existingSecret: fhir-db-secret
  existingSecretPasswordKey: password
```

#### Using Cloud-Managed Database

```bash
# For AWS RDS
externalDatabase:
  host: fhir-db.xxxxx.us-east-1.rds.amazonaws.com
  port: 5432

# For Google Cloud SQL
externalDatabase:
  host: 10.x.x.x  # Private IP via Cloud SQL Proxy
  port: 5432

# For Azure Database
externalDatabase:
  host: fhir-db.postgres.database.azure.com
  port: 5432
```

### Resource Limits

```yaml
resources:
  requests:
    cpu: 500m      # Guaranteed CPU
    memory: 512Mi  # Guaranteed memory
  limits:
    cpu: 2000m     # Maximum CPU
    memory: 2Gi    # Maximum memory
```

### Horizontal Pod Autoscaling

```yaml
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80
```

## Scaling

### Manual Scaling

```bash
# Scale deployment
kubectl scale deployment fhir-prod --replicas=10 -n fhir-production

# Verify
kubectl get pods -n fhir-production
```

### HPA Scaling

```bash
# Check HPA status
kubectl get hpa -n fhir-production

# Describe HPA
kubectl describe hpa fhir-prod -n fhir-production

# Update HPA settings
helm upgrade fhir-prod ./helm/fhir-server \
  --namespace fhir-production \
  --reuse-values \
  --set autoscaling.maxReplicas=20
```

### Load Testing

```bash
# Install k6 or similar
# Create load test script
cat <<EOF > loadtest.js
import http from 'k6/http';
import { sleep } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};

export default function () {
  http.get('https://fhir.example.com/health');
  sleep(1);
}
EOF

# Run load test
k6 run loadtest.js

# Watch HPA scale
kubectl get hpa -n fhir-production -w
```

## Monitoring

### View Logs

```bash
# All pods
kubectl logs -l app.kubernetes.io/name=fhir-server -n fhir-production -f

# Specific pod
kubectl logs fhir-prod-xxxxx-yyyyy -n fhir-production -f

# Previous instance
kubectl logs fhir-prod-xxxxx-yyyyy -n fhir-production --previous
```

### Metrics

```bash
# Pod metrics
kubectl top pods -n fhir-production

# Node metrics
kubectl top nodes

# HPA metrics
kubectl get hpa -n fhir-production
```

### Health Checks

```bash
# Check pod health
kubectl get pods -n fhir-production

# Describe pod for detailed health
kubectl describe pod fhir-prod-xxxxx-yyyyy -n fhir-production
```

## Troubleshooting

### Pods Not Starting

```bash
# Check pod status
kubectl get pods -n fhir-production

# Describe pod
kubectl describe pod fhir-prod-xxxxx-yyyyy -n fhir-production

# Check events
kubectl get events -n fhir-production --sort-by='.lastTimestamp'

# Check logs
kubectl logs fhir-prod-xxxxx-yyyyy -n fhir-production
```

### Database Connection Issues

```bash
# Exec into pod
kubectl exec -it fhir-prod-xxxxx-yyyyy -n fhir-production -- sh

# Test database connection
nc -zv postgres.example.com 5432

# Check environment variables
kubectl exec fhir-prod-xxxxx-yyyyy -n fhir-production -- env | grep DATABASE
```

### Ingress Not Working

```bash
# Check ingress
kubectl get ingress -n fhir-production
kubectl describe ingress fhir-prod-http -n fhir-production

# Check ingress controller
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller

# Test from within cluster
kubectl run curl --image=curlimages/curl -it --rm -- sh
curl http://fhir-prod-http:8080/health
```

### Certificate Issues

```bash
# Check certificate status
kubectl get certificate -n fhir-production
kubectl describe certificate fhir-server-tls -n fhir-production

# Check cert-manager logs
kubectl logs -n cert-manager deployment/cert-manager

# Manually trigger certificate renewal
kubectl delete secret fhir-server-tls -n fhir-production
```

## Maintenance

### Rolling Updates

```bash
# Update image
helm upgrade fhir-prod ./helm/fhir-server \
  --namespace fhir-production \
  --reuse-values \
  --set image.tag=1.1.0

# Watch rollout
kubectl rollout status deployment/fhir-prod -n fhir-production
```

### Rollback

```bash
# List revisions
helm history fhir-prod -n fhir-production

# Rollback to previous version
helm rollback fhir-prod -n fhir-production

# Rollback to specific revision
helm rollback fhir-prod 3 -n fhir-production
```

### Backup and Restore

```bash
# Backup Helm values
helm get values fhir-prod -n fhir-production > backup-values.yaml

# Backup database (if using bundled PostgreSQL)
kubectl exec fhir-prod-postgresql-0 -n fhir-production -- \
  pg_dump -U fhir fhir > backup.sql

# Restore
kubectl exec -i fhir-prod-postgresql-0 -n fhir-production -- \
  psql -U fhir fhir < backup.sql
```

## Best Practices

1. **Use separate namespaces** for different environments
2. **Always use resource limits** to prevent resource exhaustion
3. **Enable PodDisruptionBudget** for high availability
4. **Use external secrets management** (Vault, AWS Secrets Manager)
5. **Implement network policies** to restrict traffic
6. **Monitor your applications** with Prometheus/Grafana
7. **Regular backups** of database and configuration
8. **Test disaster recovery** procedures
9. **Use rolling updates** with proper health checks
10. **Document your deployment** and runbooks

## Additional Resources

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Helm Documentation](https://helm.sh/docs/)
- [NGINX Ingress Controller](https://kubernetes.github.io/ingress-nginx/)
- [cert-manager](https://cert-manager.io/docs/)

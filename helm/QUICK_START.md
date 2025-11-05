# Helm Chart Quick Start Guide

Get the FHIR Server running on Kubernetes in minutes!

## Prerequisites

- Docker
- kubectl
- Helm 3.x
- A Kubernetes cluster (minikube, kind, or cloud provider)

## Option 1: Local Development (minikube)

```bash
# 1. Start minikube
minikube start --cpus=4 --memory=8192

# 2. Enable ingress
minikube addons enable ingress

# 3. Build Docker image
eval $(minikube docker-env)
docker build -t fhir-server:dev .

# 4. Deploy with Helm
helm install fhir-dev ./helm/fhir-server \
  --set image.tag=dev \
  --set image.pullPolicy=Never \
  -f ./helm/fhir-server/values-development.yaml

# 5. Wait for deployment
kubectl wait --for=condition=available --timeout=300s deployment/fhir-dev

# 6. Access via port-forward
kubectl port-forward svc/fhir-dev-http 8080:8080 &
kubectl port-forward svc/fhir-dev-grpc 50051:50051 &

# 7. Test the application
curl http://localhost:8080/health
grpcurl -plaintext localhost:50051 list
```

## Option 2: Local Development (kind)

```bash
# 1. Create kind cluster
cat <<EOF | kind create cluster --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
  - containerPort: 443
    hostPort: 443
EOF

# 2. Build and load image
docker build -t fhir-server:dev .
kind load docker-image fhir-server:dev

# 3. Install NGINX Ingress
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

# 4. Wait for ingress
kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=90s

# 5. Deploy with Helm
helm install fhir-dev ./helm/fhir-server \
  --set image.tag=dev \
  --set image.pullPolicy=Never \
  -f ./helm/fhir-server/values-development.yaml

# 6. Add to /etc/hosts
echo "127.0.0.1 fhir.dev.local grpc.fhir.dev.local" | sudo tee -a /etc/hosts

# 7. Test the application
curl http://fhir.dev.local/health
grpcurl -plaintext grpc.fhir.dev.local:80 list
```

## Option 3: Production Deployment

```bash
# 1. Create namespace
kubectl create namespace fhir-production

# 2. Create secrets
kubectl create secret generic fhir-db-secret \
  --from-literal=password='your-secure-password' \
  --namespace fhir-production

kubectl create secret generic fhir-jwt-secret \
  --from-literal=jwt-secret='your-jwt-secret-key' \
  --namespace fhir-production

# 3. Build and push image to registry
docker build -t your-registry.com/fhir-server:1.0.0 .
docker push your-registry.com/fhir-server:1.0.0

# 4. Update production values
# Edit helm/fhir-server/values-production.yaml with your settings

# 5. Deploy with Helm
helm install fhir-prod ./helm/fhir-server \
  --namespace fhir-production \
  -f ./helm/fhir-server/values-production.yaml \
  --set image.repository=your-registry.com/fhir-server \
  --set image.tag=1.0.0

# 6. Verify deployment
kubectl get pods -n fhir-production
kubectl get ingress -n fhir-production

# 7. Test the application
curl https://fhir.example.com/health
```

## Common Commands

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=fhir-server

# View logs
kubectl logs -l app.kubernetes.io/name=fhir-server -f

# Port forward (for testing)
kubectl port-forward svc/fhir-server-http 8080:8080
kubectl port-forward svc/fhir-server-grpc 50051:50051

# Check HPA
kubectl get hpa

# Scale manually
kubectl scale deployment fhir-server --replicas=5

# Upgrade deployment
helm upgrade fhir-server ./helm/fhir-server --reuse-values

# Rollback
helm rollback fhir-server

# Uninstall
helm uninstall fhir-server
```

## Troubleshooting

### Pods stuck in Pending
```bash
kubectl describe pod <pod-name>
# Check for resource constraints or PVC issues
```

### Can't pull image
```bash
# For local development, ensure image exists locally
docker images | grep fhir-server

# For minikube, use minikube's docker daemon
eval $(minikube docker-env)

# For kind, load the image
kind load docker-image fhir-server:dev
```

### Database connection errors
```bash
# Check database pod
kubectl get pods -l app.kubernetes.io/name=postgresql

# Get database password
kubectl get secret fhir-server-postgresql -o jsonpath="{.data.password}" | base64 -d

# Exec into pod and test connection
kubectl exec -it <pod-name> -- psql -h fhir-server-postgresql -U fhir -d fhir
```

### Ingress not working
```bash
# Check ingress
kubectl get ingress
kubectl describe ingress fhir-server-http

# Check ingress controller
kubectl get pods -n ingress-nginx
kubectl logs -n ingress-nginx <controller-pod>
```

## Next Steps

- Read the full documentation in `helm/fhir-server/README.md`
- Check deployment guide in `docs/kubernetes_deployment.md`
- Configure monitoring and alerting
- Set up CI/CD pipeline
- Implement backup strategy

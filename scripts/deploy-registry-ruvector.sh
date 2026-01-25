#!/bin/bash
# Deploy llm-registry with Ruvector credentials
# Usage: ./scripts/deploy-registry-ruvector.sh

set -euo pipefail

PROJECT_ID="agentics-dev"
REGION="us-central1"
SERVICE_NAME="llm-registry"
IMAGE="us-central1-docker.pkg.dev/${PROJECT_ID}/llm-registry/llm-registry:ruvector"

echo "🚀 Deploying ${SERVICE_NAME} with Ruvector credentials..."

gcloud run deploy ${SERVICE_NAME} \
  --project=${PROJECT_ID} \
  --region=${REGION} \
  --image=${IMAGE} \
  --platform=managed \
  --allow-unauthenticated \
  --port=3000 \
  --cpu=2 \
  --memory=2Gi \
  --min-instances=0 \
  --max-instances=10 \
  --concurrency=80 \
  --timeout=300 \
  --cpu-boost \
  --service-account=llm-registry-sa@${PROJECT_ID}.iam.gserviceaccount.com \
  --add-cloudsql-instances=${PROJECT_ID}:${REGION}:ruvector-postgres \
  --set-env-vars="PLATFORM_ENV=production" \
  --set-env-vars="SERVICE_NAME=llm-registry" \
  --set-env-vars="SERVICE_VERSION=ruvector" \
  --set-env-vars="RUST_LOG=info" \
  --set-env-vars="SERVER_HOST=0.0.0.0" \
  --set-env-vars="SERVER_PORT=3000" \
  --set-env-vars="RUVECTOR_DB_HOST=/cloudsql/${PROJECT_ID}:${REGION}:ruvector-postgres" \
  --set-env-vars="RUVECTOR_DB_PORT=5432" \
  --set-env-vars="RUVECTOR_DB_NAME=postgres" \
  --set-env-vars="RUVECTOR_DB_USER=postgres" \
  --set-env-vars="RUVECTOR_DB_MAX_CONNECTIONS=20" \
  --set-env-vars="RUVECTOR_DB_SSL=false" \
  --set-secrets="DATABASE_URL=llm-registry-database-url:latest" \
  --set-secrets="RUVECTOR_DB_PASSWORD=RUVECTOR_DB_PASSWORD:latest" \
  --set-secrets="RUVECTOR_API_KEY=RUVECTOR_API_KEY:latest" \
  --set-secrets="RUVECTOR_SERVICE_URL=RUVECTOR_SERVICE_URL:latest" \
  --labels="app=llm-registry,env=production,team=agentics,ruvector=enabled"

echo "✅ Deployment complete!"
echo "📍 Service URL: https://${SERVICE_NAME}-1062287243982.${REGION}.run.app"

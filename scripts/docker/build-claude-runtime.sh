#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
DOCKERFILE_DIR="$ROOT_DIR/docker/claude-code"
IMAGE_NAME=${ANYON_CLOUD_IMAGE:-"anyon-claude:latest"}
BUILD_ARGS=()

if [[ -n "${CLAUDE_CODE_VERSION:-}" ]]; then
  BUILD_ARGS+=("--build-arg" "CLAUDE_CODE_VERSION=${CLAUDE_CODE_VERSION}")
fi
if [[ -n "${GH_CLI_VERSION:-}" ]]; then
  BUILD_ARGS+=("--build-arg" "GH_CLI_VERSION=${GH_CLI_VERSION}")
fi

echo "[anyon] Building image ${IMAGE_NAME}"
docker build "${DOCKERFILE_DIR}" -f "${DOCKERFILE_DIR}/Dockerfile" -t "${IMAGE_NAME}" "${BUILD_ARGS[@]}"

if [[ "${PUSH_IMAGE:-false}" == "true" ]]; then
  echo "[anyon] Pushing ${IMAGE_NAME}"
  docker push "${IMAGE_NAME}"
fi

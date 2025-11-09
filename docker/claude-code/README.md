# Claude Runtime Image

이 디렉터리는 CloudContainerService에서 사용할 런타임 이미지를 정의합니다.

## 포함 도구
- Node.js ${NODE_MAJOR:-20}
- `@anthropic-ai/claude-code` CLI
- `@github/cli`
- git/ssh/build-essential 및 비루트 `anyon` 사용자
- `/workspace` 작업 디렉터리, `/tmp/anyon-secrets` 비밀 마운트 경로

## 빌드/배포
```
ANYON_CLOUD_IMAGE=registry.example.com/anyon/claude-runtime:v0.1.0 \
CLAUDE_CODE_VERSION=2.0.31 \
GH_CLI_VERSION=2.64.0 \
PUSH_IMAGE=true \
./scripts/docker/build-claude-runtime.sh
```

환경 변수
| 변수 | 설명 |
| --- | --- |
| `ANYON_CLOUD_IMAGE` | 태그/레지스트리를 포함한 대상 이미지 이름 (기본 `anyon-claude:latest`). |
| `CLAUDE_CODE_VERSION` | 설치할 Claude Code CLI 버전 (선택). |
| `GH_CLI_VERSION` | 설치할 GitHub CLI 버전 (선택). |
| `PUSH_IMAGE` | `true`일 경우 build 후 `docker push`. |

CI에서는 동일한 스크립트 혹은 `docker build` 명령을 사용해 이미지를 생성하고, `ANYON_CLOUD_CONTAINER_IMAGE` 환경 변수로 배포 서버에 전달하면 됩니다.

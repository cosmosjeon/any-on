# Cloud Runtime Checklist

사용자가 Claude/GitHub 통합을 사용할 때 Docker 기반 CloudContainerService가 정상적으로 동작하는지 점검하기 위한 런북입니다.

## 배포 전 점검
- [ ] `ANYON_SECRET_KEY`가 base64 32바이트로 설정되어 SecretStore가 동작한다.
- [ ] `ANYON_CLOUD_CONTAINER_IMAGE`가 최신 Claude Runtime 이미지 태그를 가리킨다.
- [ ] `ANYON_TEMP_DIR`가 존재하고 권한이 700이며 충분한 디스크 공간이 있다.

## 릴리스/배포 시
1. `scripts/docker/build-claude-runtime.sh`로 이미지를 빌드하고 필요 시 registry에 push.
2. 릴리스 노트/Infra 변수에 새 태그 기록.
3. 배포 VM에서 `docker pull $ANYON_CLOUD_CONTAINER_IMAGE`로 캐시 warm-up.

## 운영 중 헬스체크
1. Settings → Integrations에서 GitHub/Claude 연결을 재현해 Claude 로그인 모달이 SSE 로그를 스트리밍하는지 확인.
2. Claude가 필요한 task attempt를 실행하고 `/tmp/anyon/cloud-secrets/<attempt>` 디렉터리가 생성됐다가 완료 후 삭제되는지 확인.
3. 문제 발생 시:
   ```bash
   docker ps --filter name=task-attempt
   docker logs <container_id>
   ```
4. Secret 디렉터리가 잔류하면 `find /tmp/anyon/cloud-secrets -maxdepth 1 -type d -mmin +60 -exec rm -rf {} +`로 정리.

## 알림/모니터링 권장 항목
- `Failed to store Claude credentials` 로그 발생 시 Slack/Sentry 알림.
- `docker exec` exit code ≠ 0, `Cloud container missing for attempt` 패턴 추적.
- Secret 디렉터리를 삭제하지 못했을 때 경고(크론 잡의 stderr → 로깅).

## 참고 문서
- `docker/claude-code/README.md` – 이미지 빌드 지침
- `docs/DEPLOYMENT.md` – Cloud env 변수 요약
- `phase-3d-web-transition-wrapup.md` – 전체 웹전환 마무리 계획

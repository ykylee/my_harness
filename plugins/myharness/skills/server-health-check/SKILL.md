---
name: server-health-check
description: Server status, logs, and deploy hygiene. Use for server status, logs, deploy, config.
---

# server-health-check

한국어로 결론과 다음 행동만 보고한다.

1. 읽기(status/logs)는 허용. 배포·설정 변경은 확인 뒤에만.
2. 프로덕션 호스트 추측으로 명령을 실행하지 않는다.
3. `rm -rf /`, 디스크 포맷, 무차별 restart 는 하지 않는다.
4. 배포는 대상 env 와 롤백 방법을 먼저 말한다.

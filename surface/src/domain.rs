//! 12 verbs → Korean engine prompt (ported from bin/myharness prompt_for).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Code,
    Server,
    Env,
}

pub fn prompt_for(domain: &str, verb: &str, rest: &[String]) -> Option<String> {
    let tail = rest.join(" ");
    let key = format!("{domain} {verb}");
    let p = match key.as_str() {
        "code review" => format!("다음 대상을 코드 리뷰하라. 한국어로 결론과 다음 행동만. 대상: {tail}"),
        "code implement" => format!("다음 기능을 구현하라. 한국어로 보고. 기능: {tail}"),
        "code test" => format!("다음 경로의 테스트를 실행하고 결과를 분석하라. 경로: {tail}"),
        "code commit" => format!("다음 메시지로 git commit 을 준비/작성하라. 메시지: {tail}"),
        "server status" => format!(
            "서버 상태를 점검하라. 호스트: {}",
            if tail.is_empty() { "local" } else { &tail }
        ),
        "server logs" => format!("서비스 로그를 분석하라. 인자: {tail}"),
        "server deploy" => format!("배포를 준비하라. 실행 전 확인이 필요하면 멈춰라. 환경: {tail}"),
        "server config" => format!("서버 설정을 조회/변경하라. 파괴적 변경은 확인 후. 동작: {tail}"),
        "env setup" => format!("다음 스택으로 환경을 부트스트랩하라. 스택: {tail}"),
        "env install" => format!("다음 패키지를 설치하라. 패키지: {tail}"),
        "env shell" => format!("다음 셸 명령을 실행하고 결과를 분석하라. 명령: {tail}"),
        "env diagnose" => {
            "현재 개발 환경을 진단하라. uname, PATH 요약, 런타임 버전. 한국어로 결론과 다음 행동만."
                .into()
        }
        _ => return None,
    };
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_has_korean_instruction() {
        let p = prompt_for("env", "diagnose", &[]).unwrap();
        assert!(p.contains("진단"));
        assert!(p.contains("한국어"));
    }
}

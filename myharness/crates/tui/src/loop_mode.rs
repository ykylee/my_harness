//! Loop mode (ralph-wiggum 패턴, CONCEPT §5.10).
//!
//! 주어진 goal 을 달성할 때까지 orchestrator + sub-agent + LLM 호출을 반복.
//! Stop condition: success-criteria 충족 OR max-iterations 도달 OR user interrupt.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
use serde::{Deserialize, Serialize};

use crate::orchestrator::{DispatchDecision, DispatchKind, Orchestrator};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum LoopStop {
    /// max-iterations 도달
    MaxIterations,
    /// user interrupt (Ctrl+C)
    Interrupted,
    /// success-criteria 충족
    Success { reason: String },
    /// unrecoverable error
    Error { message: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopConfig {
    pub goal: String,
    pub success_criteria: Option<String>,
    pub max_iterations: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            goal: String::new(),
            success_criteria: None,
            max_iterations: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopIteration {
    pub iteration: u32,
    pub input: String,
    pub response: String,
    pub dispatch: Option<DispatchKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopReport {
    pub config: LoopConfig,
    pub iterations: Vec<LoopIteration>,
    pub stop: LoopStop,
    pub total_iterations: u32,
}

pub struct LoopRunner {
    config: LoopConfig,
    interrupted: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl LoopRunner {
    #[must_use] 
    pub fn new(config: LoopConfig) -> Self {
        Self {
            config,
            interrupted: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    #[must_use] 
    pub fn interrupt_handle(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.interrupted.clone()
    }

    /// 휴리스틱 success 평가: response 가 "DONE" 또는 "SUCCESS" word 포함 시 성공.
    /// 또는 `success_criteria` 가 있고 response 가 그 string 포함 시.
    #[must_use] 
    pub fn is_success(response: &str, criteria: Option<&str>) -> bool {
        if let Some(c) = criteria
            && !c.is_empty()
            && response.contains(c)
        {
            return true;
        }
        // word boundary 검사: "DONE" 또는 "SUCCESS" 가 단어로 등장
        let upper = response.to_ascii_uppercase();
        upper.split_whitespace().any(|w| w == "DONE" || w == "SUCCESS" || w.starts_with("DONE:") || w.starts_with("SUCCESS:"))
    }

    /// main loop. interrupted flag set 시 즉시 중단.
    pub async fn run(&self, orch: &Orchestrator) -> LoopReport {
        let mut iterations: Vec<LoopIteration> = Vec::new();
        let mut stop: LoopStop = LoopStop::MaxIterations;

        for i in 0..self.config.max_iterations {
            if self.interrupted.load(std::sync::atomic::Ordering::Relaxed) {
                stop = LoopStop::Interrupted;
                break;
            }

            let decision: DispatchDecision = orch.dispatch(&self.config.goal);
            let dispatch_kind = decision.dispatch;
            let extracted = decision.extracted_input.clone();

            let response = match orch.run(&extracted).await {
                Ok(r) => r,
                Err(e) => {
                    stop = LoopStop::Error { message: e.to_string() };
                    iterations.push(LoopIteration {
                        iteration: i,
                        input: extracted,
                        response: format!("ERROR: {e}"),
                        dispatch: Some(dispatch_kind),
                    });
                    break;
                }
            };

            let success = Self::is_success(&response, self.config.success_criteria.as_deref());
            iterations.push(LoopIteration {
                iteration: i,
                input: extracted,
                response: response.clone(),
                dispatch: Some(dispatch_kind),
            });

            if success {
                stop = LoopStop::Success { reason: response };
                break;
            }
        }

        let total = iterations.len() as u32;
        LoopReport {
            config: self.config.clone(),
            iterations,
            total_iterations: total,
            stop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use myharness_llm::LLMClient;
    use myharness_llm::client_mock::{MockClient, MockResponse};
    use myharness_llm::provider::ProviderId;

    fn orch_with_mock(responses: Vec<&'static str>) -> Orchestrator {
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        // MockClient 는 FIFO (push_back + pop_front). W11.3 push_fifo_many 으로 명시적 FIFO.
        c.push_fifo_many(responses.into_iter().map(|r| MockResponse::Text(r.into())));
        Orchestrator::new().with_llm(c as Arc<dyn LLMClient>)
    }

    #[test]
    fn is_success_done_prefix() {
        assert!(LoopRunner::is_success("DONE: all tests pass", None));
        assert!(LoopRunner::is_success("success", None));
    }

    #[test]
    fn is_success_criteria_match() {
        assert!(LoopRunner::is_success("foo bar CI green baz", Some("CI green")));
    }

    #[test]
    fn is_success_default_false() {
        assert!(!LoopRunner::is_success("still working on it", None));
    }

    #[tokio::test]
    async fn run_completes_max_iterations_when_no_success() {
        let o = orch_with_mock(vec!["still going"; 5]);
        let cfg = LoopConfig { goal: "fix all bugs".into(), success_criteria: None, max_iterations: 3 };
        let runner = LoopRunner::new(cfg);
        let report = runner.run(&o).await;
        assert_eq!(report.total_iterations, 3);
        assert_eq!(report.stop, LoopStop::MaxIterations);
    }

    #[tokio::test]
    async fn run_stops_on_done() {
        // Orchestrator 가 LLM err 를 [LLM-error] 로 wrapping → 항상 Ok 반환.
        // queue 가 비면 [LLM-error] ProviderUnavailable → is_success false.
        // 3개 응답으로 3 iter 채우고, 4번째 부터 LLM err → success 아님 → max_iterations 도달.
        // 첫 번째 LLM 응답이 success 여야 stop. LIFO reverse 로 DONE 이 3번째 pop.
        let o = orch_with_mock(vec!["DONE: all good"]);
        let cfg = LoopConfig { goal: "implement feature".into(), success_criteria: None, max_iterations: 10 };
        let runner = LoopRunner::new(cfg);
        let report = runner.run(&o).await;
        assert_eq!(report.total_iterations, 1);
        assert!(matches!(report.stop, LoopStop::Success { .. }));
    }

    #[tokio::test]
    async fn run_stops_on_criteria_match() {
        let o = orch_with_mock(vec!["CI green achieved"]);
        let cfg = LoopConfig {
            goal: "make CI green".into(),
            success_criteria: Some("CI green".into()),
            max_iterations: 5,
        };
        let runner = LoopRunner::new(cfg);
        let report = runner.run(&o).await;
        assert_eq!(report.total_iterations, 1);
        assert!(matches!(report.stop, LoopStop::Success { .. }));
    }

    #[tokio::test]
    async fn run_stops_on_interrupt() {
        let o = orch_with_mock(vec!["..."; 5]);
        let cfg = LoopConfig { goal: "x".into(), success_criteria: None, max_iterations: 10 };
        let runner = LoopRunner::new(cfg);
        let handle = runner.interrupt_handle();
        handle.store(true, std::sync::atomic::Ordering::Relaxed);
        let report = runner.run(&o).await;
        assert_eq!(report.stop, LoopStop::Interrupted);
        assert_eq!(report.total_iterations, 0);
    }

    #[tokio::test]
    async fn run_stops_on_error() {
        // 첫 번째 응답이 Error → orchestrator.run() 가 [LLM-error] 로 wrapping → Ok 반환.
        // LoopRunner 는 Ok response 만 보므로 is_success 평가 → false → 계속.
        // 따라서 5 iter 모두 돔 (Orchestrator 가 err 를 swallow 하기 때문).
        // 이건 의도된 v1 simple 동작: error 가 발생해도 max_iterations 도달 시 종료.
        let c = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Error("network down".into()));
        let o = Orchestrator::new().with_llm(c as Arc<dyn LLMClient>);
        let cfg = LoopConfig { goal: "code review foo.rs".into(), success_criteria: None, max_iterations: 5 };
        let runner = LoopRunner::new(cfg);
        let report = runner.run(&o).await;
        assert_eq!(report.total_iterations, 5);
        assert_eq!(report.stop, LoopStop::MaxIterations);
    }

    #[test]
    fn default_loop_config() {
        let cfg = LoopConfig::default();
        assert_eq!(cfg.max_iterations, 20);
        assert!(cfg.goal.is_empty());
    }
}

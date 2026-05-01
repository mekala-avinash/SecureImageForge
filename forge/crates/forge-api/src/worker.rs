use std::time::Duration;

use forge_core::audit::Outcome;
use forge_core::domain::BuildStatus;
use forge_core::team::BuildQueueRepo;

use crate::routes::record_audit_with_details;
use crate::state::ApiState;

pub fn start_workers(state: ApiState) -> Vec<tokio::task::JoinHandle<()>> {
    let mut handles = Vec::new();
    let concurrency = state.config.workers.concurrency.max(1);
    for idx in 0..concurrency {
        let state = state.clone();
        handles.push(tokio::spawn(async move {
            let worker_id = format!("worker-{idx}");
            loop {
                if let Err(e) = run_one(&state, &worker_id).await {
                    tracing::warn!(error = %e, worker = %worker_id, "worker iteration failed");
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }));
    }
    handles
}

async fn run_one(state: &ApiState, worker_id: &str) -> anyhow::Result<()> {
    if !state.config.features.durable_queue {
        return Ok(());
    }
    let queue: &std::sync::Arc<dyn BuildQueueRepo> = &state.queue;
    let lease = queue
        .lease_next(worker_id, state.config.workers.lease_seconds)
        .await?;
    let Some(job) = lease else {
        return Ok(());
    };

    queue.mark_running(&job.id).await?;
    let build_id = uuid::Uuid::parse_str(&job.build_id)?;
    let record = state
        .builds
        .get_record_in_project(&job.project_id, build_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing build for job {}", job.id))?;
    let orchestrator = state.orchestrator().await;
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    let state_clone = state.clone();
    
    let poller = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if let Ok(Some(summary)) = state_clone.builds.get_summary(build_id).await {
                if summary.status == "cancelled" {
                    cancel_token_clone.cancel();
                    break;
                }
            }
        }
    });

    let result = tokio::select! {
        res = orchestrator.run_existing(record) => {
            poller.abort();
            res
        }
        _ = cancel_token.cancelled() => {
            poller.abort();
            Err(forge_core::Error::Internal(anyhow::anyhow!("build cancelled by user")))
        }
    };
    match result {
        Ok(outcome) => {
            if outcome.record.status == BuildStatus::Succeeded {
                queue.mark_success(&job.id).await?;
                record_audit_with_details(
                    &state.audit,
                    "queue-worker",
                    "job.succeeded",
                    Some(&job.id),
                    Outcome::Success,
                    Some(serde_json::json!({"build_id": job.build_id, "worker_id": worker_id})),
                )
                .await;
            } else {
                let status = queue
                    .mark_failure_retry_or_deadletter(
                        &job.id,
                        "build failed",
                        backoff_seconds(job.attempts, &state.config.workers.backoff_strategy),
                    )
                    .await?;
                record_audit_with_details(
                    &state.audit,
                    "queue-worker",
                    "job.failed",
                    Some(&job.id),
                    Outcome::Error,
                    Some(serde_json::json!({"build_id": job.build_id, "status": status, "worker_id": worker_id})),
                )
                .await;
            }
        }
        Err(e) => {
            let status = queue
                .mark_failure_retry_or_deadletter(
                    &job.id,
                    &e.to_string(),
                    backoff_seconds(job.attempts, &state.config.workers.backoff_strategy),
                )
                .await?;
            record_audit_with_details(
                &state.audit,
                "queue-worker",
                "job.error",
                Some(&job.id),
                Outcome::Error,
                Some(serde_json::json!({"error": e.to_string(), "status": status, "worker_id": worker_id})),
            )
            .await;
        }
    }
    Ok(())
}

fn backoff_seconds(attempts: i64, strategy: &str) -> u64 {
    if strategy == "fixed" {
        return 5;
    }
    let attempts = attempts.max(1) as u32;
    2u64.saturating_pow(attempts.min(6))
}

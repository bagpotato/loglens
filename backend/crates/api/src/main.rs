use std::{sync::Arc, time::Duration};

use anyhow::Context;
use loglens_alerts::{Dedup, NtfyNotifier, Poller};
use loglens_api::{build_router, AppState};
use loglens_auth::JwtKeys;
use loglens_config::Settings;
use loglens_core::LogStore;
use loglens_loki::LokiStore;
use loglens_severity::Classifier;
use tokio::{net::TcpListener, task::JoinHandle};
use tracing_subscriber::EnvFilter;

const ALERT_POLL_EVERY: Duration = Duration::from_secs(30);
const ALERT_DEDUP_WINDOW: Duration = Duration::from_secs(10 * 60);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let settings = Settings::from_env().context("invalid configuration")?;
    let pool = loglens_storage::connect(&settings.db_url)
        .await
        .context("failed to open database")?;
    let admin_password = settings.admin_password.as_deref();
    if loglens_auth::ensure_admin(&pool, &settings.admin_user, admin_password)
        .await
        .context("failed to create admin user")?
    {
        tracing::info!(user = %settings.admin_user, "created admin user");
    }

    let store: Arc<dyn LogStore> = Arc::new(LokiStore::new(&settings.loki_url));
    let ai = loglens_ai::build_provider(&settings.ai);
    let classifier =
        Arc::new(Classifier::new(pool.clone(), ai.clone()).with_masking(settings.ai.mask));
    let alerts = spawn_alerts(&settings, &store, &pool);

    let state = AppState {
        pool,
        jwt: JwtKeys::new(&settings.jwt_secret),
        store,
        ai,
        classifier,
        ai_mask: settings.ai.mask,
    };
    let listener = TcpListener::bind(&settings.bind)
        .await
        .with_context(|| format!("failed to bind {}", settings.bind))?;
    tracing::info!(addr = %settings.bind, "listening");
    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    if let Some(handle) = alerts {
        handle.abort();
    }
    tracing::info!("stopped");
    Ok(())
}

fn spawn_alerts(
    settings: &Settings,
    store: &Arc<dyn LogStore>,
    pool: &loglens_storage::SqlitePool,
) -> Option<JoinHandle<()>> {
    let Some(ntfy) = settings.ntfy.as_ref() else {
        tracing::info!("LOGLENS_NTFY_URL not set, alerts disabled");
        return None;
    };
    let notifier = Arc::new(NtfyNotifier::new(ntfy));
    let dedup = Dedup::new(pool.clone(), ALERT_DEDUP_WINDOW);
    tracing::info!(url = %ntfy.url, topic = %ntfy.topic, "alerts enabled");
    Some(Poller::new(store.clone(), notifier, dedup).spawn(ALERT_POLL_EVERY))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::warn!(error = %err, "failed to listen for ctrl-c");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutting down");
}

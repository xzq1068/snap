use std::fs::create_dir_all;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn init_log(app_path: &PathBuf) -> anyhow::Result<WorkerGuard> {
    let log_dir = app_path.join("logs");

    if !log_dir.exists() {
        create_dir_all(&log_dir)?;
    }

    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (non_block_appender, _guard) = tracing_appender::non_blocking(file_appender);

    //文件输出层
    let file_layer = fmt::layer()
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
        .with_writer(non_block_appender)
        .with_ansi(false)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true);

    //控制台输出层
    let console_layer = fmt::layer()
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_level(true)
        .with_target(false);

    //组合所有层
    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    Ok(_guard)
}

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn init_logger(max_level: Level) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(max_level)
        //.with_writer(rolling::daily("logs", "processed_tx.log"))
        //.with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

use std::io::prelude::*;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = BoxedError> = core::result::Result<T, E>;

/// Have producer generate content at roughly this rate.
const BITS_PER_SEC: usize = 6_000_000;

enum Mode {
    Producer,
    StdConsumer,
    TokioConsumer,
}

fn usage_error() -> ! {
    eprintln!("Usage: tokio-stdin --producer | tokio-stdin [--tokio-consumer|--std-consumer]");
    std::process::exit(-1);
}

#[tokio::main]
async fn main() {
    let arg = std::env::args().skip(1).next();
    let mode = match arg.as_ref().map(|s| s.as_str()) {
        Some("--producer") => Mode::Producer,
        Some("--tokio-consumer") => Mode::TokioConsumer,
        Some("--std-consumer") => Mode::StdConsumer,
        _ => usage_error(),
    };

    match mode {
        Mode::Producer => producer().await,
        Mode::TokioConsumer => tokio_consumer().await,
        Mode::StdConsumer => std_consumer().await,
    }
    .unwrap();
}

async fn producer() -> Result<()> {
    const INTERVAL: Duration = Duration::from_secs(1);

    let ts_buffer = [b'G'; 188 * 1000];
    let dest = std::io::stdout();
    let mut dest = dest.lock();

    loop {
        let start = Instant::now();
        let mut bytes_written = 0usize;
        tokio::task::block_in_place(|| -> Result<()> {
            while bytes_written < BITS_PER_SEC / 8 {
                dest.write_all(&ts_buffer)?;
                bytes_written += ts_buffer.len();
            }
            Ok(())
        })?;
        // tokio::task::block_in_place(|| dest.flush())?;

        let duration = (start + INTERVAL) - Instant::now();
        tokio::time::sleep(duration).await;
    }
}

async fn tokio_consumer() -> Result<()> {
    let mut src = tokio::io::stdin();

    let mut ts_buffer = [0u8; 188];
    loop {
        src.read_exact(&mut ts_buffer).await?;
    }
}

async fn std_consumer() -> Result<()> {
    let src = std::io::stdin();
    let mut src = src.lock();

    let mut ts_buffer = [0u8; 188 * 8];
    loop {
        // tokio::task::block_in_place(|| src.read_exact(&mut ts_buffer))?;
        src.read_exact(&mut ts_buffer)?;
    }
}

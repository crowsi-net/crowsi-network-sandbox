//! One-shot playground bridge over the managed process owner, never an approval authority.
use crowsi_network_sandbox::{IsolationBudget, isolated_workload};
use crowsi_process_adapter::OwnedProcess;
use std::{ffi::OsString, io::Read, path::Path, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("runtime and secret-free input directory required".into());
    }
    let mut request = vec![];
    std::io::stdin().take(4097).read_to_end(&mut request)?;
    if request.len() > 4096 {
        return Err("worker request budget exceeded".into());
    }
    let launch = isolated_workload(
        Path::new(&args[0]),
        Path::new(&args[1]),
        &[OsString::from("/work/input/worker.mjs")],
        IsolationBudget::Short,
    )?;
    let mut worker = OwnedProcess::spawn(launch).await?;
    let mut input = worker.take_stdin().ok_or("worker input unavailable")?;
    input.write_all(&request).await?;
    input.shutdown().await?;
    drop(input);
    let mut output = worker
        .take_stdout()
        .ok_or("worker output unavailable")?
        .take(4097);
    let mut errors = worker
        .take_stderr()
        .ok_or("worker error stream unavailable")?
        .take(4097);
    let mut data = vec![];
    let mut diagnostics = vec![];
    let copied = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(
            output.read_to_end(&mut data),
            errors.read_to_end(&mut diagnostics)
        )
    })
    .await;
    if !matches!(&copied, Ok(Ok(_))) || data.len() > 4096 || diagnostics.len() > 4096 {
        worker.shutdown(Duration::from_millis(100)).await?;
        return Err("worker deadline or output budget exceeded".into());
    }
    copied??;
    let finished = tokio::time::timeout(Duration::from_secs(2), worker.wait()).await;
    if finished.is_err() {
        worker.shutdown(Duration::from_millis(100)).await?;
        return Err("worker exit deadline exceeded".into());
    }
    if finished??.code() != Some(0) {
        return Err("isolated worker failed".into());
    }
    // Diagnostics are untrusted and never printed as an authority result.
    std::io::Write::write_all(&mut std::io::stdout(), &data)?;
    Ok(())
}

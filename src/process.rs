use crate::{Error, Level, Logger};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

/// Run a command without a shell, capturing output and enforcing its deadline.
///
/// A zero duration disables the timeout. Output goes to private temporary files
/// rather than pipes, so a noisy child cannot block waiting for a pipe reader.
/// A timed-out child is killed and reaped before this function returns.
pub fn run_command(
    command: &mut Command,
    timeout: Duration,
    logger: &dyn Logger,
) -> Result<(), Error> {
    let program = std::path::PathBuf::from(command.get_program());
    logger.log(Level::Info, &format!("Executing {command:?}"));
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?)
        .spawn()?;
    let status = if timeout.is_zero() {
        child.wait()
    } else {
        match child.wait_timeout(timeout) {
            Ok(Some(status)) => Ok(status),
            Ok(None) => {
                let _ = child.kill();
                child.wait()?;
                return Err(Error::Timeout { program, timeout });
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        }
    }?;
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    stdout.read_to_end(&mut stdout_bytes)?;
    stderr.read_to_end(&mut stderr_bytes)?;
    let stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
    let stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();
    if !status.success() {
        return Err(Error::Process {
            program,
            status: status.code(),
            stdout,
            stderr,
        });
    }
    logger.log(
        Level::Info,
        &format!("Process successfully ended with output `{stdout}`"),
    );
    Ok(())
}

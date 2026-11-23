use spatie_image_optimizer::{
    DummyLogger, Error, Image, Level, Logger, Optimizer, OptimizerChain, OptimizerChainFactory,
    OptimizerStatus, OutputStatus, Tool, run_command,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn content_detection_and_svg_extension_match_php() -> Result<(), Box<dyn std::error::Error>> {
    for (name, mime, tools) in [
        ("image.jpg", "image/jpeg", vec![Tool::Jpegoptim]),
        ("logo.png", "image/png", vec![Tool::Pngquant, Tool::Optipng]),
        ("graph.svg", "image/svg+xml", vec![Tool::Svgo]),
        ("animated.gif", "image/gif", vec![Tool::Gifsicle]),
        ("image.webp", "image/webp", vec![Tool::Cwebp]),
        ("image.avif", "image/avif", vec![Tool::Avifenc]),
        ("test.txt", "text/plain", vec![]),
    ] {
        let image = Image::new(fixture(name))?;
        assert_eq!(image.mime()?, mime, "{name}");
        for tool in [
            Tool::Jpegoptim,
            Tool::Pngquant,
            Tool::Optipng,
            Tool::Svgo,
            Tool::Gifsicle,
            Tool::Cwebp,
            Tool::Avifenc,
        ] {
            assert_eq!(
                tool.with_options(Vec::<String>::new()).can_handle(&image),
                tools.contains(&tool),
                "{name} {tool:?}"
            );
        }
    }
    let directory = tempfile::tempdir()?;
    let renamed = directory.path().join("misleading.PNG");
    fs::copy(fixture("image.jpg"), &renamed)?;
    let image = Image::new(&renamed)?;
    assert_eq!(image.path(), renamed);
    assert_eq!(image.extension(), "png");
    assert!(
        Tool::Jpegoptim
            .with_options(["--strip-all"])
            .can_handle(&image)
    );
    fs::copy(fixture("graph.svg"), &renamed)?;
    assert!(
        !Tool::Svgo
            .with_options(Vec::<String>::new())
            .can_handle(&image)
    );
    let url = url::Url::from_file_path(&renamed).unwrap();
    assert_eq!(Image::new(url.as_str())?.path(), renamed);
    for invalid in [
        "https://example.com/a.png",
        "php://filter",
        "file://remote-host/a.png",
        "missing.jpg",
    ] {
        assert!(Image::new(invalid).is_err());
    }
    Ok(())
}

struct Append(&'static [u8]);
impl Optimizer for Append {
    fn name(&self) -> &str {
        "append"
    }
    fn can_handle(&self, _: &Image) -> bool {
        true
    }
    fn optimize(&self, image: &Image, logger: &dyn Logger, _: Duration) -> Result<(), Error> {
        use std::io::Write;
        logger.log(Level::Info, "custom optimizer");
        fs::OpenOptions::new()
            .append(true)
            .open(image.path())?
            .write_all(self.0)?;
        Ok(())
    }
}

struct Failure;
impl Optimizer for Failure {
    fn name(&self) -> &str {
        "failure"
    }
    fn can_handle(&self, _: &Image) -> bool {
        true
    }
    fn optimize(&self, _: &Image, _: &dyn Logger, _: Duration) -> Result<(), Error> {
        Err(Error::Custom("failed deliberately".into()))
    }
}

struct Skipped;
impl Optimizer for Skipped {
    fn name(&self) -> &str {
        "skipped"
    }
    fn can_handle(&self, _: &Image) -> bool {
        false
    }
    fn optimize(&self, _: &Image, _: &dyn Logger, _: Duration) -> Result<(), Error> {
        panic!("An unsupported optimizer must not run")
    }
}

struct Replace(&'static [u8]);
impl Optimizer for Replace {
    fn name(&self) -> &str {
        "replace"
    }
    fn can_handle(&self, _: &Image) -> bool {
        true
    }
    fn optimize(&self, image: &Image, _: &dyn Logger, _: Duration) -> Result<(), Error> {
        fs::write(image.path(), self.0)?;
        Ok(())
    }
}

#[test]
fn safe_output_accepts_only_smaller_candidates() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source.txt");
    let output = directory.path().join("output.txt");
    for (candidate, expected, status, saved) in [
        (&b"tiny"[..], &b"tiny"[..], OutputStatus::Written, 4),
        (
            &b"changed!"[..],
            &b"original"[..],
            OutputStatus::KeptOriginal,
            0,
        ),
        (
            &b"much larger output"[..],
            &b"original"[..],
            OutputStatus::KeptOriginal,
            0,
        ),
    ] {
        let chain = OptimizerChain::new()
            .safe_output(true)
            .add_optimizer(Replace(candidate));
        fs::write(&source, b"original")?;
        let report = chain.optimize_with_report(&source)?;
        assert_eq!(fs::read(&source)?, expected);
        assert_eq!(report.status, status);
        assert_eq!(report.original_bytes, 8);
        assert_eq!(report.candidate_bytes, Some(candidate.len() as u64));
        assert_eq!(report.bytes_saved(), saved);
        assert_eq!(report.input, source);
        assert_eq!(report.output, source);

        fs::write(&source, b"original")?;
        for existing in [false, true] {
            if existing {
                fs::write(&output, b"previous destination")?;
            }
            let report = chain.optimize_to_with_report(&source, &output)?;
            assert_eq!(fs::read(&source)?, b"original");
            assert_eq!(fs::read(&output)?, expected);
            assert_eq!(report.status, status);
            assert_eq!(report.candidate_bytes, Some(candidate.len() as u64));
            assert_eq!(report.bytes_saved(), saved);
            assert_eq!(report.output, output);
            fs::remove_file(&output)?;
        }
        assert_eq!(
            fs::read_dir(directory.path())?.count(),
            1,
            "scratch directory leaked"
        );
    }
    Ok(())
}

#[test]
fn safe_output_discards_failed_chains_and_cleans_up() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source.txt");
    let output = directory.path().join("output.txt");
    fs::write(&source, b"original")?;
    let chain = OptimizerChain::new()
        .safe_output(true)
        .add_optimizer(Replace(b"x"))
        .add_optimizer(Failure)
        .add_optimizer(Append(b"y"));
    let report = chain.optimize_with_report(&source)?;
    assert_eq!(fs::read(&source)?, b"original");
    assert_eq!(report.status, OutputStatus::Discarded);
    assert_eq!(report.candidate_bytes, Some(2));
    assert_eq!(report.bytes_saved(), 0);
    assert_eq!(
        report.optimizers[1].status,
        OptimizerStatus::Failed("failed deliberately".into())
    );
    assert_eq!(report.optimizers[2].status, OptimizerStatus::Succeeded);
    let report = chain.optimize_to_with_report(&source, &output)?;
    assert_eq!(report.status, OutputStatus::Discarded);
    assert!(!output.exists());
    fs::write(&output, b"previous output")?;
    let handled = chain.on_error(|_, _, image| {
        assert_ne!(image.path().file_name().unwrap(), "source.txt");
        Ok(())
    });
    assert_eq!(
        handled.optimize_to_with_report(&source, &output)?.status,
        OutputStatus::Discarded
    );
    assert_eq!(fs::read(&output)?, b"previous output");
    let strict = handled.throws(true);
    assert!(matches!(
        strict.optimize_to(&source, &output),
        Err(Error::Custom(_))
    ));
    assert_eq!(fs::read(&source)?, b"original");
    assert_eq!(fs::read(&output)?, b"previous output");
    assert!(
        strict
            .on_error(|_, _, _| Err(Error::Custom("stop".into())))
            .optimize(&source)
            .is_err()
    );
    assert_eq!(fs::read(&source)?, b"original");
    assert_eq!(fs::read_dir(directory.path())?.count(), 2);
    Ok(())
}

#[test]
fn reports_include_skips_failures_and_growth_without_safe_output()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source.txt");
    fs::write(&source, b"original")?;
    let report = OptimizerChain::new()
        .safe_output(true)
        .safe_output(false)
        .add_optimizer(Skipped)
        .add_optimizer(Append(b"-continued"))
        .add_optimizer(Failure)
        .optimize_with_report(&source)?;
    assert_eq!(fs::read(&source)?, b"original-continued");
    assert_eq!(report.status, OutputStatus::Written);
    assert_eq!(report.original_bytes, 8);
    assert_eq!(report.candidate_bytes, Some(18));
    assert_eq!(report.bytes_saved(), 0);
    assert_eq!(
        report
            .optimizers
            .iter()
            .map(|r| (r.name.as_str(), &r.status))
            .collect::<Vec<_>>(),
        vec![
            ("skipped", &OptimizerStatus::Skipped),
            ("append", &OptimizerStatus::Succeeded),
            (
                "failure",
                &OptimizerStatus::Failed("failed deliberately".into())
            ),
        ]
    );
    assert!(
        report
            .optimizers
            .iter()
            .map(|r| r.elapsed)
            .sum::<Duration>()
            <= report.elapsed
    );
    let report = OptimizerChain::new()
        .safe_output(true)
        .add_optimizer(Skipped)
        .optimize_with_report(&source)?;
    assert_eq!(report.status, OutputStatus::KeptOriginal);
    Ok(())
}

#[cfg(unix)]
#[test]
fn safe_output_preserves_modes_and_rejects_destination_symlinks()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source.txt");
    let output = directory.path().join("output.txt");
    fs::write(&source, b"original")?;
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640))?;
    let chain = OptimizerChain::new()
        .safe_output(true)
        .add_optimizer(Replace(b"tiny"));
    chain.optimize_to(&source, &output)?;
    assert_eq!(fs::metadata(&output)?.permissions().mode() & 0o777, 0o640);
    fs::set_permissions(&output, fs::Permissions::from_mode(0o600))?;
    chain.optimize_to(&source, &output)?;
    assert_eq!(fs::metadata(&output)?.permissions().mode() & 0o777, 0o600);
    let alias = directory.path().join("alias.txt");
    fs::hard_link(&source, &alias)?;
    assert!(chain.optimize_to(&source, &alias).is_err());
    chain.optimize(&source)?;
    assert_eq!(fs::read(&alias)?, b"original");
    assert_eq!(fs::read(&source)?, b"tiny");
    assert_eq!(fs::metadata(&source)?.permissions().mode() & 0o777, 0o640);
    let link = directory.path().join("link.txt");
    symlink(&source, &link)?;
    assert!(matches!(chain.optimize(&link), Err(Error::InvalidPath(_))));
    assert!(matches!(
        chain.optimize_to(&alias, &link),
        Err(Error::InvalidPath(_))
    ));
    fs::remove_file(&link)?;
    symlink(directory.path().join("absent"), &link)?;
    assert!(matches!(
        chain.optimize_to(&source, &link),
        Err(Error::InvalidPath(_))
    ));
    assert!(!directory.path().join("absent").exists());
    Ok(())
}

#[test]
fn ordered_custom_optimizers_and_error_modes() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source.txt");
    fs::write(&source, b"start")?;
    let logs = Arc::new(Mutex::new(Vec::new()));
    let sink = logs.clone();
    let chain = OptimizerChain::new()
        .add_optimizer(Skipped)
        .add_optimizer(Append(b"-one"))
        .add_optimizer(Failure)
        .add_optimizer(Append(b"-two"))
        .use_logger(move |level, message: &str| {
            sink.lock().unwrap().push((level, message.to_owned()));
        });
    chain.optimize(&source)?;
    assert_eq!(fs::read(&source)?, b"start-one-two");
    assert!(
        logs.lock()
            .unwrap()
            .iter()
            .any(|(level, text)| *level == Level::Error && text.contains("failed deliberately"))
    );
    let strict = chain.throws(true);
    assert!(matches!(strict.optimize(&source), Err(Error::Custom(_))));
    assert_eq!(fs::read(&source)?, b"start-one-two-one");
    strict.throws(false).optimize(&source)?;
    assert_eq!(fs::read(&source)?, b"start-one-two-one-one-two");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let captured = seen.clone();
    OptimizerChain::new()
        .add_optimizer(Failure)
        .add_optimizer(Append(b"-handled"))
        .on_error(move |error, optimizer, image| {
            captured.lock().unwrap().push((
                error.to_string(),
                optimizer.name().to_owned(),
                image.path().to_owned(),
            ));
            Ok(())
        })
        .optimize(&source)?;
    assert_eq!(
        seen.lock().unwrap()[0],
        (
            "failed deliberately".into(),
            "failure".into(),
            source.clone()
        )
    );
    let before = fs::read(&source)?;
    let abort = OptimizerChain::new()
        .add_optimizer(Failure)
        .add_optimizer(Append(b"-unexpected"))
        .on_error(|_, _, _| Err(Error::Custom("aborted by handler".into())));
    assert!(
        matches!(abort.optimize(&source), Err(Error::Custom(message)) if message == "aborted by handler")
    );
    assert_eq!(fs::read(&source)?, before);
    Ok(())
}

#[test]
fn output_copy_validation_and_chain_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source.txt");
    let output = directory.path().join("output.txt");
    fs::write(&source, b"original")?;
    let chain = OptimizerChain::new()
        .add_optimizer(Append(b"-discarded"))
        .set_optimizers(vec![Box::new(Append(b"-kept"))]);
    assert_eq!(chain.get_optimizers().len(), 1);
    chain.optimize_to(&source, &output)?;
    assert_eq!(fs::read(&source)?, b"original");
    assert_eq!(fs::read(&output)?, b"original-kept");
    assert!(chain.optimize_to(&source, &source).is_err());
    let alias = directory.path().join("alias.txt");
    fs::hard_link(&source, &alias)?;
    assert!(chain.optimize_to(&source, &alias).is_err());
    assert_eq!(fs::read(&source)?, b"original");
    assert!(matches!(
        chain.optimize_to(&source, directory.path().join("missing/output.txt")),
        Err(Error::Copy { .. })
    ));
    assert!(
        chain
            .optimize_to(&source, "https://example.com/out.png")
            .is_err()
    );
    assert!(OptimizerChainFactory::with_quality(101).is_err());
    assert!(OptimizerChainFactory::with_quality(0).is_ok());
    assert!(OptimizerChainFactory::with_quality(100).is_ok());
    let names: Vec<_> = OptimizerChainFactory::create()
        .get_optimizers()
        .iter()
        .map(|optimizer| optimizer.name().to_owned())
        .collect();
    assert_eq!(
        names,
        [
            "jpegoptim",
            "pngquant",
            "optipng",
            "svgo",
            "gifsicle",
            "cwebp",
            "avifenc"
        ]
    );
    OptimizerChainFactory::create()
        .throws(true)
        .optimize(&source)?;
    assert_eq!(fs::read(&source)?, b"original");
    Ok(())
}

#[test]
fn builds_commands_with_literal_arguments_and_custom_paths() {
    let optimizer = Tool::Cwebp
        .with_options(["-q", "20"])
        .set_options(["-q", "50"])
        .set_binary_path("tools folder")
        .set_binary_name("custom-webp");
    assert_eq!(optimizer.tool(), Tool::Cwebp);
    assert_eq!(optimizer.binary_name(), "custom-webp");
    assert_eq!(optimizer.options(), ["-q", "50"]);
    let image = Path::new("photos/it's a ; tricky.webp");
    let command = optimizer.command(image, image);
    assert_eq!(
        command.get_program(),
        Path::new("tools folder").join("custom-webp")
    );
    let args: Vec<_> = command.get_args().collect();
    assert_eq!(
        args,
        [
            "-q".as_ref(),
            "50".as_ref(),
            image.as_os_str(),
            "-o".as_ref(),
            image.as_os_str()
        ]
    );
}

fn child(action: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "process_fixture", "--nocapture"])
        .env("SPATIE_TEST_PROCESS", action);
    command
}

struct ProcessOptimizer;
impl Optimizer for ProcessOptimizer {
    fn name(&self) -> &str {
        "process"
    }
    fn can_handle(&self, _: &Image) -> bool {
        true
    }
    fn optimize(&self, _: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error> {
        run_command(&mut child("sleep"), timeout, logger)
    }
}

#[test]
fn process_fixture() {
    match std::env::var("SPATIE_TEST_PROCESS").as_deref() {
        Ok("fail") => {
            print!("diagnostic stdout");
            eprint!("diagnostic stderr");
            std::process::exit(17);
        }
        Ok("sleep") => std::thread::sleep(Duration::from_secs(20)),
        Ok("noisy") => {
            print!("{}", "x".repeat(256 * 1024));
            eprint!("{}", "y".repeat(256 * 1024));
        }
        _ => (),
    }
}

#[test]
fn process_failures_capture_output_and_deadlines_do_not_deadlock()
-> Result<(), Box<dyn std::error::Error>> {
    let error = run_command(&mut child("fail"), Duration::from_secs(5), &DummyLogger).unwrap_err();
    assert!(
        matches!(error, Error::Process { status: Some(17), stdout, stderr, .. }
        if stdout.contains("diagnostic stdout") && stderr.contains("diagnostic stderr"))
    );
    let started = Instant::now();
    assert!(matches!(
        run_command(
            &mut child("sleep"),
            Duration::from_millis(100),
            &DummyLogger
        ),
        Err(Error::Timeout { .. })
    ));
    assert!(started.elapsed() < Duration::from_secs(5));
    run_command(&mut child("noisy"), Duration::from_secs(5), &DummyLogger)?;
    run_command(&mut child("noisy"), Duration::ZERO, &DummyLogger)?;
    let directory = tempfile::tempdir()?;
    let file = directory.path().join("timeout.txt");
    fs::write(&file, b"original")?;
    let chain = OptimizerChain::new()
        .set_timeout(Duration::from_millis(100))
        .add_optimizer(ProcessOptimizer)
        .add_optimizer(Append(b"-continued"));
    chain.optimize(&file)?;
    assert_eq!(fs::read(&file)?, b"original-continued");
    let chain = chain.safe_output(true);
    let report = chain.optimize_with_report(&file)?;
    assert_eq!(report.status, OutputStatus::Discarded);
    assert!(
        matches!(&report.optimizers[0].status, OptimizerStatus::Failed(message) if message.contains("exceeded"))
    );
    assert_eq!(fs::read(&file)?, b"original-continued");
    assert!(matches!(
        chain.throws(true).optimize(&file),
        Err(Error::Timeout { .. })
    ));
    assert_eq!(fs::read(&file)?, b"original-continued");
    assert_eq!(fs::read_dir(directory.path())?.count(), 1);
    Ok(())
}

#[cfg(unix)]
#[test]
fn avif_reserves_and_cleans_intermediate_before_error_callback()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let decoder = directory.path().join("check-reserved-and-fail");
    fs::write(
        &decoder,
        "#!/bin/sh\nfor output; do :; done\ntest -f \"$output\" || exit 19\nprintf '%s' \"$output\" >&2\nexit 17\n",
    )?;
    fs::set_permissions(&decoder, fs::Permissions::from_mode(0o700))?;
    let image = directory.path().join("source.avif");
    fs::copy(fixture("image.avif"), &image)?;
    OptimizerChain::new()
        .add_optimizer(
            Tool::Avifenc
                .with_options(Vec::<String>::new())
                .set_decode_binary_name(decoder.as_os_str()),
        )
        .on_error(|error, _, _| match error {
            Error::Process {
                status: Some(17),
                stderr,
                ..
            } => {
                assert!(stderr.ends_with(".png"));
                assert!(!Path::new(stderr).exists());
                Ok(())
            }
            other => panic!("Expected reservation check to pass, got {other}"),
        })
        .optimize(&image)?;
    assert_eq!(fs::read(image)?, fs::read(fixture("image.avif"))?);
    Ok(())
}

#[test]
fn missing_binaries_follow_chain_error_policy() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let image = directory.path().join("image.jpg");
    fs::copy(fixture("image.jpg"), &image)?;
    let chain = OptimizerChain::new()
        .add_optimizer(
            Tool::Jpegoptim
                .with_options(["--strip-all"])
                .set_binary_name("spatie-nonexistent-optimizer-test"),
        )
        .add_optimizer(Append(b"continued"));
    chain.optimize(&image)?;
    assert!(fs::read(&image)?.ends_with(b"continued"));
    assert!(matches!(
        chain.throws(true).optimize(&image),
        Err(Error::Io(_))
    ));
    Ok(())
}

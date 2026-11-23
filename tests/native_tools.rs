//! These tests require all seven native tools. CI installs them in a dedicated Linux job.
#[cfg(unix)]
use spatie_image_optimizer::{Error, OptimizerChain, Tool};
use spatie_image_optimizer::{Image, OptimizerChainFactory, OutputStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
#[ignore = "requires jpegoptim, pngquant, optipng, svgo, gifsicle, cwebp, avifdec and avifenc"]
fn optimizes_every_format_and_preserves_originals() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    for name in [
        "image.jpg",
        "logo.png",
        "graph.svg",
        "animated.gif",
        "image.webp",
        "image.avif",
    ] {
        let source = fixture(name);
        let original = fs::read(&source)?;
        let output = directory.path().join(format!("it's a ; {name}"));
        let report = OptimizerChainFactory::create()
            .safe_output(true)
            .throws(true)
            .optimize_to_with_report(&source, &output)?;
        let optimized = fs::read(&output)?;
        eprintln!("{name}: {} -> {} bytes", original.len(), optimized.len());
        assert!(optimized.len() < original.len(), "{name} did not shrink");
        assert_eq!(report.status, OutputStatus::Written);
        assert_eq!(report.original_bytes, original.len() as u64);
        assert_eq!(report.candidate_bytes, Some(optimized.len() as u64));
        assert_eq!(
            report.bytes_saved(),
            (original.len() - optimized.len()) as u64
        );
        assert_eq!(report.optimizers.len(), 7);
        assert_eq!(Image::new(&output)?.mime()?, Image::new(&source)?.mime()?);
        assert_eq!(fs::read(&source)?, original, "{name} source changed");
        if name.ends_with("avif") {
            assert!(
                Command::new("avifdec")
                    .arg("--info")
                    .arg(&output)
                    .status()?
                    .success()
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires jpegoptim"]
fn quality_affects_real_encoder_output() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let low = directory.path().join("low.jpg");
    let high = directory.path().join("high.jpg");
    OptimizerChainFactory::with_quality(20)?
        .throws(true)
        .optimize_to(fixture("image.jpg"), &low)?;
    OptimizerChainFactory::with_quality(95)?
        .throws(true)
        .optimize_to(fixture("image.jpg"), &high)?;
    assert!(fs::metadata(low)?.len() < fs::metadata(high)?.len());
    Ok(())
}

#[cfg(unix)]
#[test]
#[ignore = "requires avifdec"]
fn avif_temporary_output_is_cleaned_on_encoder_failure() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let encoder = directory.path().join("record-and-fail");
    fs::write(&encoder, "#!/bin/sh\nprintf '%s' \"$1\" >&2\nexit 17\n")?;
    fs::set_permissions(&encoder, fs::Permissions::from_mode(0o700))?;
    let original = fs::read(fixture("image.avif"))?;
    let chain = OptimizerChain::new().throws(true).add_optimizer(
        Tool::Avifenc
            .with_options(Vec::<String>::new())
            .set_binary_name(encoder.as_os_str()),
    );
    match chain.optimize(fixture("image.avif")) {
        Err(Error::Process {
            status: Some(17),
            stderr,
            ..
        }) => {
            assert!(stderr.ends_with(".png"));
            assert!(
                !Path::new(&stderr).exists(),
                "temporary decode output leaked: {stderr}"
            );
        }
        other => panic!("expected encoder failure, got {other:?}"),
    }
    assert_eq!(fs::read(fixture("image.avif"))?, original);
    Ok(())
}

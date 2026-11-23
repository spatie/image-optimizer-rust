//! Executes the Markdown examples verbatim, with application context outside the snippets.
use spatie_image_optimizer::{Error, Image, Level, Logger, Optimizer, Tool};
use std::time::Duration;

struct MyLogger;

impl Logger for MyLogger {
    fn log(&self, level: Level, message: &str) {
        eprintln!("{level:?}: {message}");
    }
}

struct YourCustomOptimizer;

impl Optimizer for YourCustomOptimizer {
    fn name(&self) -> &str {
        "custom-jpegoptim"
    }

    fn can_handle(&self, image: &Image) -> bool {
        Tool::Jpegoptim
            .with_options(["--strip-all"])
            .can_handle(image)
    }

    fn optimize(&self, image: &Image, logger: &dyn Logger, timeout: Duration) -> Result<(), Error> {
        Tool::Jpegoptim
            .with_options(["--strip-all"])
            .optimize(image, logger, timeout)
    }
}

fn report(exception: &Error) {
    eprintln!("{exception}");
}

macro_rules! documentation_example {
    ($name:ident, ($chain:ident, $input:ident, $output:ident), { $($code:tt)* }) => {
        #[test]
        #[ignore = "requires the native image tools; CI runs this target in the native-tools job"]
        #[allow(unused_imports, unused_variables, dead_code)]
        fn $name() -> Result<(), Error> {
            let directory = tempfile::tempdir()?;
            let source = directory.path().join("image.jpg");
            let destination = directory.path().join("image-optimized.jpg");
            std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/image.jpg"), &source)?;
            let $input = source.as_path();
            let $output = destination.as_path();
            let $chain = spatie_image_optimizer::OptimizerChainFactory::create();

            $($code)*

            assert_eq!(Image::new(&source)?.mime()?, "image/jpeg");
            Ok(())
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/documentation.rs"));

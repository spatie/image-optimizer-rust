//! Runs the self-contained Markdown examples, including rustdoc's hidden setup.
macro_rules! documentation_example {
    ($name:ident, { $($code:tt)* }) => {
        #[test]
        #[ignore = "requires the native image tools; CI runs this target in the native-tools job"]
        #[allow(unused_imports, unused_variables, dead_code)]
        fn $name() -> Result<(), spatie_image_optimizer::Error> {
            $($code)*
            main()
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/documentation.rs"));

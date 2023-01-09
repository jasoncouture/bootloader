use bootloader_test_runner::run_test_kernel;

#[test]
fn basic_boot() {
    run_test_kernel(
        env!("CARGO_BIN_FILE_TEST_KERNEL_HIGHER_HALF_basic_boot"),
        None,
    );
}

#[test]
fn should_panic() {
    run_test_kernel(
        env!("CARGO_BIN_FILE_TEST_KERNEL_HIGHER_HALF_should_panic"),
        None,
    );
}

#[test]
fn check_boot_info() {
    run_test_kernel(
        env!("CARGO_BIN_FILE_TEST_KERNEL_HIGHER_HALF_check_boot_info"),
        None,
    );
}

#[test]
fn verify_higher_half() {
    run_test_kernel(
        env!("CARGO_BIN_FILE_TEST_KERNEL_HIGHER_HALF_verify_higher_half"),
        None,
    );
}

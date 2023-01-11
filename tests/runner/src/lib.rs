use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

const QEMU_ARGS: &[&str] = &[
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial",
    "stdio",
    "-display",
    "none",
    "--no-reboot",
];

pub fn run_test_kernel_with_ramdisk(kernel_binary_path: &str, ramdisk_path: Option<PathBuf>) {
    let kernel_path = Path::new(kernel_binary_path);
    let kernel_path_buf = kernel_path.to_path_buf();
    let ramdisk_path_buf = ramdisk_path.as_ref();
    run_test_kernel_with_ramdisk_internal(&kernel_path_buf, ramdisk_path_buf)
}

fn run_test_kernel_with_ramdisk_internal(
    kernel_path_buf: &PathBuf,
    ramdisk_path_buf: Option<&PathBuf>,
) {
    use bootloader::DiskImageBuilder;

    let mut image_builder = DiskImageBuilder::new(&kernel_path_buf);

    // We don't use if/let here because the borrow checker doesn't see the outer lifetime, and we need it to last at least
    // as long as DiskImageBuilder does. So instead, take ownership and unrwap the parameter.
    if ramdisk_path_buf.is_some() {
        image_builder.set_ramdisk(ramdisk_path_buf.to_owned().unwrap());
    }

    #[cfg(feature = "bios")]
    {
        let mbr_path = kernel_path_buf.with_extension("mbr");
        image_builder.create_bios_image(&mbr_path).unwrap();
        run_test_kernel_on_bios(&mbr_path);
    }

    #[cfg(feature = "uefi")]
    {
        let gpt_path = kernel_path_buf.with_extension("gpt");
        let tftp_path = kernel_path_buf.with_extension("tftp");
        image_builder.create_uefi_image(&gpt_path).unwrap();
        image_builder.create_uefi_tftp_folder(&tftp_path).unwrap();
        run_test_kernel_on_uefi(&gpt_path);
        run_test_kernel_on_uefi_pxe(&tftp_path);
    }
}

pub fn run_test_kernel(kernel_binary_path: &str) {
    run_test_kernel_with_ramdisk(kernel_binary_path, None);
}

#[cfg(feature = "uefi")]
pub fn run_test_kernel_on_uefi(out_gpt_path: &Path) {
    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-drive")
        .arg(format!("format=raw,file={}", out_gpt_path.display()));
    run_cmd.args(QEMU_ARGS);
    run_cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    let child_output = run_cmd.output().unwrap();
    strip_ansi_escapes::Writer::new(std::io::stderr())
        .write_all(&child_output.stderr)
        .unwrap();
    strip_ansi_escapes::Writer::new(std::io::stderr())
        .write_all(&child_output.stdout)
        .unwrap();

    match child_output.status.code() {
        Some(33) => {}                     // success
        Some(35) => panic!("Test failed"), // success
        other => panic!("Test failed with unexpected exit code `{:?}`", other),
    }
}

#[cfg(feature = "bios")]
pub fn run_test_kernel_on_bios(out_mbr_path: &Path) {
    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-drive")
        .arg(format!("format=raw,file={}", out_mbr_path.display()));
    run_cmd.args(QEMU_ARGS);

    let child_output = run_cmd.output().unwrap();
    strip_ansi_escapes::Writer::new(std::io::stderr())
        .write_all(&child_output.stderr)
        .unwrap();
    strip_ansi_escapes::Writer::new(std::io::stderr())
        .write_all(&child_output.stdout)
        .unwrap();

    match child_output.status.code() {
        Some(33) => {}                     // success
        Some(35) => panic!("Test failed"), // success
        other => panic!("Test failed with unexpected exit code `{:?}`", other),
    }
}

#[cfg(feature = "uefi")]
pub fn run_test_kernel_on_uefi_pxe(out_tftp_path: &Path) {
    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd.arg("-netdev").arg(format!(
        "user,id=net0,net=192.168.17.0/24,tftp={},bootfile=bootloader,id=net0",
        out_tftp_path.display()
    ));
    run_cmd.arg("-device").arg("virtio-net-pci,netdev=net0");
    run_cmd.args(QEMU_ARGS);
    run_cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    let child_output = run_cmd.output().unwrap();
    strip_ansi_escapes::Writer::new(std::io::stderr())
        .write_all(&child_output.stderr)
        .unwrap();
    strip_ansi_escapes::Writer::new(std::io::stderr())
        .write_all(&child_output.stdout)
        .unwrap();

    match child_output.status.code() {
        Some(33) => {} // success
        Some(35) => panic!("Test failed"),
        other => panic!("Test failed with unexpected exit code `{:?}`", other),
    }
}

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const RUN_ARGS: &[&str] = &["--no-reboot", "-s"];
const TEST_ARGS: &[&str] = &[
    "-device",
    "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial",
    "stdio",
    "-display",
    "none",
    "--no-reboot",
];
const TEST_TIMEOUT_SECS: u64 = 10;

fn main() {
    let mut args = env::args().skip(1); // skip executable name
    let kernel_binary_path = PathBuf::from(args.next().unwrap()).canonicalize().unwrap();
    let bios = create_disk_images(&kernel_binary_path);

    let no_boot = match args.next() {
        Some(arg) => match arg.as_str() {
            "--no-run" => true,
            other => panic!("unexpected argument `{}`", other),
        },
        None => false,
    };

    if no_boot {
        println!("Created disk image at `{}`", bios.display());
        return;
    }

    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd.args([
        "-drive",
        format!("format=raw,file={}", bios.display()).as_str(),
        #[cfg(any(unix))]
        "--enable-kvm",
    ]);

    match runner_utils::binary_kind(&kernel_binary_path).is_test() {
        true => test(run_cmd),
        false => run(run_cmd),
    };
}

fn run(mut run_cmd: Command) {
    run_cmd.args(RUN_ARGS);

    let exit_status = run_cmd.status().unwrap();
    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }

}

fn test(mut run_cmd: Command) {
    run_cmd.args(TEST_ARGS);

    let exit_status = runner_utils::run_with_timeout(&mut run_cmd, Duration::from_secs(TEST_TIMEOUT_SECS)).unwrap();
    match exit_status.code() {
        Some(33) => return, // success
        other => panic!("Test failed (exit code: {:?})", other),
    }
}

pub fn create_disk_images(kernel_binary_path: &Path) -> PathBuf {
    let bootloader_manifest_path = bootloader_locator::locate_bootloader("bootloader").unwrap();
    let kernel_manifest_path = locate_cargo_manifest::locate_manifest().unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest_path.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest_path);
    build_cmd.arg("--kernel-binary").arg(&kernel_binary_path);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest_path.parent().unwrap().join("target"));
    build_cmd
        .arg("--out-dir")
        .arg(kernel_binary_path.parent().unwrap());
    build_cmd.arg("--quiet");

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }

    let kernel_binary_name = kernel_binary_path.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_binary_path
        .parent()
        .unwrap()
        .join(format!("boot-bios-{}.img", kernel_binary_name));

    if !disk_image.exists() {
        panic!(
            "Disk image does not exist at {} after bootloader build",
            disk_image.display()
        );
    }

    disk_image
}

arch := env_var_or_default("KERNEL_ARCH", "x86_64")
profile := env_var_or_default("KERNEL_PROFILE", "dev")
qemu_flags := env_var_or_default("QEMU_FLAGS", "-m 2G")
cargo_flags := env_var_or_default("CARGO_FLAGS", "--features qemu-exit")
profile_subdir := if profile == "dev" { "debug" } else { profile }
out_path := "./target/target-" + arch + "/" + profile_subdir
iso_path := "./target/kernel-" + arch + "-" + profile + ".iso"
limine_config := if profile == "dev" { "limine-dev.conf" } else { "limine.conf" }

# [doc("Build the kernel for the given architecture. Available: 'x86_64'.")]
build-kernel:
    cargo build --target target-{{ arch }}.json --profile {{ profile }} {{ cargo_flags }}

# [doc("Build the ISO image using limine and the built kernel.")]
build-iso: build-kernel
    mkdir -p {{ out_path }}/boot
    mkdir -p {{ out_path }}/boot/limine
    mkdir -p {{ out_path }}/EFI/BOOT

    # Copy kernel into ISO image
    cp {{ out_path }}/kernel {{ out_path }}/boot/kernel

    # Copy Limine files
    cp ./limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin {{ out_path }}/boot/limine/
    cp ./limine/BOOTX64.EFI {{ out_path }}/EFI/BOOT/
    cp ./limine/BOOTIA32.EFI {{ out_path }}/EFI/BOOT/
    cp {{ limine_config }} {{ out_path }}/boot/limine/limine.conf

    # Create iso file
    xorriso -as mkisofs \
      -b boot/limine/limine-uefi-cd.bin \
      --efi-boot boot/limine/limine-uefi-cd.bin \
      -efi-boot-part --efi-boot-image --protective-msdos-label \
      {{ out_path }} -o {{ iso_path }}

    ./limine/limine bios-install {{ iso_path }}

# [doc("Get ovmf files for UEFI booting.")]
get-ovmf:
    # Create ovmf directory
    mkdir -p ./target/ovmf
    # Download ovmf code & vars
    curl -Lo ./target/ovmf/ovmf-code-{{ arch }}.fd https://github.com/osdev0/edk2-ovmf-nightly/releases/download/nightly-20251203T012444Z/ovmf-code-{{ arch }}.fd
    curl -Lo ./target/ovmf/ovmf-vars-{{ arch }}.fd https://github.com/osdev0/edk2-ovmf-nightly/releases/download/nightly-20251203T012444Z/ovmf-vars-{{ arch }}.fd

# [doc("Run the kernel ISO in QEMU.")]
qemu: get-ovmf build-iso
    qemu-system-{{ arch }} \
        -M q35 \
        -drive if=pflash,unit=0,format=raw,file=./target/ovmf/ovmf-code-{{ arch }}.fd,readonly=on \
        -drive if=pflash,unit=1,format=raw,file=./target/ovmf/ovmf-vars-{{ arch }}.fd \
        -cdrom {{ iso_path }} \
        -serial stdio \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
        {{ qemu_flags }}

# [doc("Update the ISO with the new kernel binary.")]
update-iso: build-kernel
    xorriso -dev {{ iso_path }} -boot_image any keep -update {{ out_path }}/kernel /boot/kernel -commit

# [doc("Build the kernel, move it into the ISO and run qemu, instead of recreating the ISO.")]
fast-qemu: update-iso
    @just --no-deps qemu

# [doc("Clean the target directory.")]
clean:
    cargo clean

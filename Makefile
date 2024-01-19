all: Cargo.toml src/main.rs
	cargo b --release --target x86_64-unknown-uefi

disk: all
	cp target/x86_64-unknown-uefi/release/befiboot.efi disk/efi/boot/BOOTX64.EFI

qemu: disk
	qemu-system-x86_64 -enable-kvm -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd -drive format=raw,file=fat:rw:disk


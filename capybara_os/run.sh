#!/bin/bash

pushd `pwd`

cd bootloader

cargo build --target x86_64-unknown-uefi
cd ..
mkdir -p esp/efi/boot

cp bootloader/target/x86_64-unknown-uefi/debug/bootloader.efi esp/efi/boot/bootx64.efi

exec qemu-system-x86_64 \
	-enable-kvm \
	-drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.4m.fd \
	-drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.4m.fd \
	-drive format=raw,file=fat:rw:esp

popd

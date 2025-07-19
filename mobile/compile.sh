rm -rf GBCEmulatorMobile
 ./build-rust.sh
 swift-bridge-cli create-package \
--bridges-dir ./generated \
--out-dir GBCEmulatorMobile \
--ios target/aarch64-apple-ios/release/libgbc_plus_mobile.a \
--simulator target/universal-ios/release/libgbc_plus_mobile.a \
--macos target/universal-macos/release/libgbc_plus_mobile.a \
--name GBCEmulatorMobile
./gbc-emu.sh
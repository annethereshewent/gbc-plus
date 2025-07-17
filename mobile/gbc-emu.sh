cd GBCEmulatorMobile

sed -i '' "s/RustXcframework/RustXcframework3/g" Package.swift

cd Sources/gbcEmulatorMobile

sed -i '' "s/RustXcframework/RustXcframework3/g" gbc-plus-mobile.swift
sed -i '' "s/RustXcframework/RustXcframework3/g" SwiftBridgeCore.swift

cd ../..

mv RustXcframework.xcframework RustXcframework3.xcframework

cd RustXcframework3.xcframework/ios-arm64/Headers

sed -i '' "s/RustXcframework/RustXcframework3/g" module.modulemap

mkdir gbc-plus
mv gbc-plus-mobile.h ./gbc-plus/gbc-plus-mobile.h
mv module.modulemap ./gbc-plus/module.modulemap
mv SwiftBridgeCore.h ./gbc-plus/SwiftBridgeCore.h

cd ../..

cd ios-arm64_x86_64-simulator/Headers

sed -i '' "s/RustXcframework/RustXcframework3/g" module.modulemap

mkdir gbc-plus
mv gbc-plus-mobile.h ./gbc-plus/gbc-plus-mobile.h
mv module.modulemap ./gbc-plus/module.modulemap
mv SwiftBridgeCore.h ./gbc-plus/SwiftBridgeCore.h

cd ../..

cd macos-arm64_x86_64/headers

sed -i '' "s/RustXcframework/RustXcframework3/g" module.modulemap

mkdir gbc-plus
mv gbc-plus-mobile.h ./gbc-plus/gbc-plus-mobile.h
mv module.modulemap ./gbc-plus/module.modulemap
mv SwiftBridgeCore.h ./gbc-plus/SwiftBridgeCore.h


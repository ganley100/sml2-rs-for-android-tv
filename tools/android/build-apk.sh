#!/usr/bin/env bash
# Builds the Android TV APK (armeabi-v7a) for Google TV sticks such as the onn.
#
#   ANDROID_HOME=~/Android/Sdk tools/android/build-apk.sh
#
# Needs: rustup target armv7-linux-androideabi, an Android SDK with build-tools
# and a platform >= 29, the NDK (ANDROID_NDK_HOME or $ANDROID_HOME/ndk/*),
# a JDK (keytool), python3 and patch.
#
# Signing: SML2_ANDROID_KEYSTORE / SML2_ANDROID_KEYSTORE_PASSWORD / SML2_ANDROID_KEY_ALIAS
# choose the key. Without them a local key is created once in ~/.android and
# reused, so later builds install over earlier ones.
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
HERE="$ROOT/tools/android"
OUT="$ROOT/target/android"
TARGET=armv7-linux-androideabi
ABI=armeabi-v7a
MIN_SDK=26
# 29 keeps legacy shared storage, so the ROM browser can open Download.
TARGET_SDK=29
PROFILE=${SML2_ANDROID_PROFILE:-release}

die() { echo "build-apk: $*" >&2; exit 1; }
newest() { ls -d "$@" 2>/dev/null | sort -V | tail -n 1; }

SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}
[[ -n "$SDK" && -d "$SDK" ]] || die "set ANDROID_HOME to your Android SDK"
NDK=${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-${ANDROID_NDK_LATEST_HOME:-$(newest "$SDK"/ndk/*)}}}
[[ -n "$NDK" && -d "$NDK" ]] || die "no NDK found; set ANDROID_NDK_HOME"
BUILD_TOOLS=$(newest "$SDK"/build-tools/*)
[[ -n "$BUILD_TOOLS" ]] || die "no SDK build-tools installed"
PLATFORM=$(newest "$SDK"/platforms/android-*)
[[ -n "$PLATFORM" && -f "$PLATFORM/android.jar" ]] || die "no SDK platform installed"
LLVM=$(newest "$NDK"/toolchains/llvm/prebuilt/*)/bin
[[ -d "$LLVM" ]] || die "NDK has no LLVM toolchain at $LLVM"

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)
IFS=. read -r MAJOR MINOR PATCH <<<"${VERSION%%-*}"
VERSION_CODE=${SML2_ANDROID_VERSION_CODE:-$((MAJOR * 10000 + MINOR * 100 + PATCH))}

mkdir -p "$OUT"

# 1. winit with the gamepad patch (controller buttons, D-pad hat and stick).
WINIT_SRC=$(cd "$ROOT" && cargo metadata --format-version 1 --filter-platform "$TARGET" \
    | python3 -c 'import json,sys,os; m=json.load(sys.stdin); print(next(os.path.dirname(p["manifest_path"]) for p in m["packages"] if p["name"]=="winit"))')
WINIT="$OUT/winit"
rm -rf "$WINIT"
cp -R "$WINIT_SRC" "$WINIT"
chmod -R u+w "$WINIT"
for p in "$HERE"/patches/*.patch; do
    patch -d "$WINIT" -p1 --quiet <"$p" || die "patch $(basename "$p") no longer applies to $(basename "$WINIT_SRC")"
done

# 2. The native library. The [patch] override rewrites Cargo.lock; restore it.
cp "$ROOT/Cargo.lock" "$OUT/Cargo.lock.orig"
trap 'cp "$OUT/Cargo.lock.orig" "$ROOT/Cargo.lock"' EXIT
LINKER="$LLVM/armv7a-linux-androideabi${MIN_SDK}-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$LINKER"
export CC_armv7_linux_androideabi="$LINKER"
export CXX_armv7_linux_androideabi="$LINKER++"
export AR_armv7_linux_androideabi="$LLVM/llvm-ar"
PROFILE_FLAG=--release
[[ "$PROFILE" == release ]] || PROFILE_FLAG="--profile=$PROFILE"
(cd "$ROOT" && cargo build "$PROFILE_FLAG" --target "$TARGET" -p sml2-android \
    --config "patch.crates-io.winit.path=\"$WINIT\"")
PROFILE_DIR=$PROFILE
[[ "$PROFILE" == dev ]] && PROFILE_DIR=debug
SO="$ROOT/target/$TARGET/$PROFILE_DIR/libsml2_android.so"
[[ -f "$SO" ]] || die "cargo did not produce $SO"

# 3. Resources, manifest and the stripped library.
STAGE="$OUT/stage"
rm -rf "$STAGE" && mkdir -p "$STAGE/lib/$ABI"
"$LLVM/llvm-strip" --strip-unneeded -o "$STAGE/lib/$ABI/libsml2_android.so" "$SO"
"$BUILD_TOOLS/aapt2" compile --dir "$HERE/res" -o "$OUT/res.zip"
"$BUILD_TOOLS/aapt2" link -o "$OUT/unaligned.apk" \
    -I "$PLATFORM/android.jar" \
    --manifest "$HERE/AndroidManifest.xml" \
    --min-sdk-version "$MIN_SDK" --target-sdk-version "$TARGET_SDK" \
    --version-code "$VERSION_CODE" --version-name "$VERSION" \
    "$OUT/res.zip"
python3 - "$OUT/unaligned.apk" "$STAGE" <<'PY'
import os, sys, zipfile
apk, stage = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(apk, "a", zipfile.ZIP_DEFLATED) as z:
    for base, _, files in os.walk(os.path.join(stage, "lib")):
        for name in files:
            path = os.path.join(base, name)
            z.write(path, os.path.relpath(path, stage))
PY

# 4. Align and sign.
KEYSTORE=${SML2_ANDROID_KEYSTORE:-$HOME/.android/sml2-tv.keystore}
STOREPASS=${SML2_ANDROID_KEYSTORE_PASSWORD:-android}
ALIAS=${SML2_ANDROID_KEY_ALIAS:-sml2}
if [[ ! -f "$KEYSTORE" ]]; then
    mkdir -p "$(dirname "$KEYSTORE")"
    keytool -genkeypair -keystore "$KEYSTORE" -storepass "$STOREPASS" -keypass "$STOREPASS" \
        -alias "$ALIAS" -keyalg RSA -keysize 2048 -validity 10000 \
        -dname "CN=Six Golden Coins, O=sml2-rs" >/dev/null
    echo "build-apk: created signing key $KEYSTORE (keep it to update the app later)"
fi
APK="$OUT/six-golden-coins-androidtv-$ABI.apk"
"$BUILD_TOOLS/zipalign" -p -f 4 "$OUT/unaligned.apk" "$OUT/aligned.apk"
"$BUILD_TOOLS/apksigner" sign --ks "$KEYSTORE" --ks-pass "pass:$STOREPASS" \
    --ks-key-alias "$ALIAS" --out "$APK" "$OUT/aligned.apk"
"$BUILD_TOOLS/apksigner" verify "$APK"
rm -f "$OUT/unaligned.apk" "$OUT/aligned.apk" "$APK.idsig"
echo "build-apk: $APK"

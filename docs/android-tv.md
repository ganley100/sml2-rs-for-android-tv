# Android TV / Google TV

Six Golden Coins runs on Google TV and Android TV devices, including the
**onn. streaming sticks**, as a 32-bit ARM (`armeabi-v7a`) app. It needs
Android 8.0 or newer and appears in the TV's app row with its own banner.

## Get the APK

- **GitHub Actions:** the **Android TV build** workflow builds
  `six-golden-coins-androidtv-armeabi-v7a.apk` as a run artifact on every push
  that touches the game, and tagged releases attach it to the release.
- **Build it yourself:** see [Building](#building) below.

## Install on the TV

1. On the TV open **Settings → System → About** and press **OK** seven times
   on **Android TV OS build** to enable developer options.
2. Install a sideloading app such as **Downloader** or **Send Files to TV**
   from the Play Store, then allow it under **Settings → Apps → Security &
   restrictions → Unknown sources**.
3. Open the APK with that app and choose **Install**. Over USB debugging or
   network ADB you can instead run `adb install six-golden-coins-androidtv-armeabi-v7a.apk`.

## Add your ROM

ROMs are not included. Copy your **Super Mario Land 2 UE v1.0** `.gb` file
(and optionally the SML2DX ROM or IPS) to the TV's **Download** folder, for
example with Send Files to TV or `adb push sml2.gb /sdcard/Download/`.

Launch the app and allow storage access when asked; the ROM browser opens in
Download. Choose **Settings → Game → Import ROM…**, pick the file, then
**Play**. If you decline storage access the browser opens the app's own folder,
`/sdcard/Android/data/io.github.ganley100.sixgoldencoins/files`, which `adb push`
can fill without the permission.

Settings, saves and the imported game data stay in the app's private storage.

## Controls

A Bluetooth or USB game controller is recommended; the TV remote can drive the
menus but has no jump button.

| Action | Controller | TV remote |
| --- | --- | --- |
| Move / navigate | D-pad or left stick | D-pad |
| Jump / confirm | A | |
| Run / fire | B or X | |
| Spin jump (in the air) | L2 / R2 | |
| Game Start | Left stick click | Select (center) |
| Game Select | Select / Back / View | |
| Pause menu / back | Start / Menu | Back |

Controller buttons can be rebound in **Settings → Controls**. Press **Home** to
leave the game.

## Building

Requirements: `rustup target add armv7-linux-androideabi`, an Android SDK with
build-tools and a platform of API 29 or newer, the Android NDK, a JDK (for
`keytool`), `python3` and `patch`.

```bash
ANDROID_HOME=~/Android/Sdk tools/android/build-apk.sh
# -> target/android/six-golden-coins-androidtv-armeabi-v7a.apk
```

The script:

1. copies winit from the Cargo registry and applies
   `tools/android/patches/winit-android-gamepad.patch`, which forwards
   controller buttons and the D-pad hat / left stick (winit drops them on
   Android) as F13–F32 key presses that `input::android_button` maps back to
   pad buttons;
2. builds `crates/sml2-android` (a `NativeActivity` cdylib) against that copy,
   restoring `Cargo.lock` afterwards;
3. packages `tools/android/AndroidManifest.xml` and `tools/android/res`
   (launcher icon and the 320×180 TV banner) with `aapt2`, then aligns and signs.

The APK targets API 29 so the ROM browser can read the shared Download folder.

**Signing.** Set `SML2_ANDROID_KEYSTORE`, `SML2_ANDROID_KEYSTORE_PASSWORD` and
`SML2_ANDROID_KEY_ALIAS` to use your own key. Otherwise the script creates
`~/.android/sml2-tv.keystore` once and reuses it. Android only installs an
update signed with the same key, so keep it; CI reads the same values from the
`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD` and `ANDROID_KEY_ALIAS`
repository secrets.

## Limitations

- Only one controller is read at a time, and the Settings controller picker
  does not list Android controllers.
- The DX import and file browser are the in-app ones; Android has no system
  file dialog here.

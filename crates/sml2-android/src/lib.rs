//! Android TV entry point. `NativeActivity` loads this library and calls
//! [`android_main`]; the desktop app then runs fullscreen on the TV.
#![cfg(target_os = "android")]

use sml2_desktop::app::{self, Options};
use std::path::{Path, PathBuf};
use winit::platform::android::activity::AndroidApp;

#[no_mangle]
fn android_main(android: AndroidApp) {
    if let Err(error) = run(android) {
        eprintln!("sml2-android: {error}");
    }
}

fn run(android: AndroidApp) -> Result<(), String> {
    let internal = android
        .internal_data_path()
        .ok_or("Android did not provide an app data folder")?;
    point_user_dirs_at(&internal);
    if let Some(external) = android.external_data_path() {
        let _ = std::fs::create_dir_all(&external);
        std::env::set_var("SML2_EXTERNAL_DIR", &external);
    }
    // Asset discovery and relative paths resolve against the app's own folder.
    let _ = std::env::set_current_dir(&internal);
    if let Err(error) = request_storage_permission() {
        eprintln!("sml2-android: storage permission request failed: {error}");
    }

    let mut options = Options::defaults()?;
    options.fullscreen = true;
    options.finish_launch(false);
    app::run_with(
        options,
        eframe::NativeOptions {
            android_app: Some(android),
            ..Default::default()
        },
    )
}

/// `directories` reads `HOME` and the XDG variables on Android; point them at
/// private app storage so settings, saves and the imported ROM cache persist.
fn point_user_dirs_at(internal: &Path) {
    let dir = |name: &str| -> PathBuf {
        let path = internal.join(name);
        let _ = std::fs::create_dir_all(&path);
        path
    };
    std::env::set_var("HOME", internal);
    std::env::set_var("XDG_CONFIG_HOME", dir("config"));
    std::env::set_var("XDG_DATA_HOME", dir("data"));
    std::env::set_var("XDG_CACHE_HOME", dir("cache"));
}

/// Asks once for shared-storage read access so the ROM browser can open the
/// Download folder. Declining still leaves the app's own external folder.
fn request_storage_permission() -> Result<(), jni::errors::Error> {
    use jni::objects::{JObject, JValue};

    const PERMISSION: &str = "android.permission.READ_EXTERNAL_STORAGE";
    const GRANTED: i32 = 0;
    let context = ndk_context::android_context();
    // SAFETY: android-activity initialises the context with the process's VM
    // and a global reference to the running activity before `android_main`.
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }?;
    let activity = unsafe { JObject::from_raw(context.context().cast()) };
    let mut env = vm.attach_current_thread()?;
    let permission = env.new_string(PERMISSION)?;
    let state = env
        .call_method(
            &activity,
            "checkSelfPermission",
            "(Ljava/lang/String;)I",
            &[JValue::Object(&permission)],
        )?
        .i()?;
    if state != GRANTED {
        let request = env.new_object_array(1, "java/lang/String", &permission)?;
        env.call_method(
            &activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[JValue::Object(&request), JValue::Int(1)],
        )?;
    }
    Ok(())
}

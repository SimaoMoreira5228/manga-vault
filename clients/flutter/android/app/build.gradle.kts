plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val repoRoot = layout.projectDirectory.dir("../../../..")

data class Abi(val jni: String, val cargoTarget: String, val clangPrefix: String)

val androidAbis = listOf(
    Abi("arm64-v8a", "aarch64-linux-android", "aarch64-linux-android"),
    Abi("armeabi-v7a", "armv7-linux-androideabi", "armv7a-linux-androideabi"),
    Abi("x86_64", "x86_64-linux-android", "x86_64-linux-android"),
)

fun ndkBinDir(): File {
    val sdk = System.getenv("ANDROID_HOME")
        ?: System.getenv("ANDROID_SDK_ROOT")
        ?: error("ANDROID_HOME is not set")
    val ndk = File("$sdk/ndk").listFiles()?.maxByOrNull { it.name }
        ?: error("no NDK found under $sdk/ndk")
    val host = when {
        System.getProperty("os.name").lowercase().contains("windows") -> "windows-x86_64"
        System.getProperty("os.name").lowercase().contains("mac") -> "darwin-x86_64"
        else -> "linux-x86_64"
    }
    return File(ndk, "toolchains/llvm/prebuilt/$host/bin")
}

fun abiEnvironment(abi: Abi): Map<String, String> {
    val bin = ndkBinDir().absolutePath
    val suffix = if (System.getProperty("os.name").lowercase().contains("windows")) ".cmd" else ""
    val upper = abi.cargoTarget.uppercase().replace('-', '_')
    val lower = abi.cargoTarget.replace('-', '_')
    return mapOf(
        "CARGO_TARGET_${upper}_LINKER" to "$bin/${abi.clangPrefix}24-clang$suffix",
        "CC_${lower}" to "$bin/${abi.clangPrefix}24-clang$suffix",
        "AR_${lower}" to "$bin/llvm-ar$suffix",
    )
}

androidAbis.forEach { abi ->
    tasks.register<Exec>("buildCoreFfi${abi.jni.replace("-", "")}") {
        workingDir(repoRoot)
        commandLine("cargo", "build", "-p", "core-ffi", "--target", abi.cargoTarget)
        environment(abiEnvironment(abi))
        inputs.dir(repoRoot.dir("crates"))
        inputs.file(repoRoot.file("Cargo.toml"))
        outputs.file(repoRoot.file("target/${abi.cargoTarget}/debug/libcore_ffi.so"))
    }
}

tasks.register<Copy>("installCoreFfiJniLibs") {
    into(layout.projectDirectory.dir("src/main/jniLibs").asFile)
    androidAbis.forEach { abi ->
        into(abi.jni) {
            from(repoRoot.file("target/${abi.cargoTarget}/debug/libcore_ffi.so"))
        }
    }
    dependsOn(androidAbis.map { tasks.named("buildCoreFfi${it.jni.replace("-", "")}") })
}

tasks.named("preBuild") {
    dependsOn("installCoreFfiJniLibs")
}



android {
    namespace = "dev.mangavault.manga_vault"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "dev.mangavault.manga_vault"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}

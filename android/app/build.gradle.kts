plugins {
    id("com.android.application")
}

android {
    namespace = "tr.tulpar.kalkan"
    compileSdk = 35

    defaultConfig {
        applicationId = "tr.tulpar.kalkan"
        minSdk = 29 // Android 10+ (RoleManager & CallScreeningService tam uyumu)
        targetSdk = 35
        versionCode = 100
        versionName = "1.0.0-tulpar"

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.webkit:webkit:1.11.0")
}

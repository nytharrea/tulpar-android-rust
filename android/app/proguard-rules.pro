# Tulpar JNI & WebView JS Bridge kuralları
-keep class tr.tulpar.kalkan.RustBridge { *; }
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}

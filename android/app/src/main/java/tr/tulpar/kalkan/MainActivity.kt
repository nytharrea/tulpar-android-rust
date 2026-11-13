package tr.tulpar.kalkan

import android.Manifest
import android.annotation.SuppressLint
import android.app.role.RoleManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.webkit.JavascriptInterface
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Rust Motorunu Başlat
        RustBridge.initEngine(filesDir.absolutePath)

        webView = WebView(this)
        setContentView(webView)

        webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            cacheMode = WebSettings.LOAD_NO_CACHE
            allowFileAccess = true
        }

        webView.addJavascriptInterface(TulparJsBridge(this), "TulparNative")
        webView.loadUrl("file:///android_asset/index.html")
    }

    inner class TulparJsBridge(private val context: Context) {

        @JavascriptInterface
        fun getEngineState(): String {
            return RustBridge.getStateJson()
        }

        @JavascriptInterface
        fun saveEngineState(newJson: String): String {
            return RustBridge.updateConfigJson(newJson)
        }

        @JavascriptInterface
        fun testSms(sender: String, body: String): String {
            return RustBridge.inspectSmsJson(sender, body, false)
        }

        @JavascriptInterface
        fun syncUsomList(): String {
            return RustBridge.syncUsomNow()
        }

        @JavascriptInterface
        fun requestModeAPermissions() {
            ActivityCompat.requestPermissions(
                this@MainActivity,
                arrayOf(
                    Manifest.permission.RECEIVE_SMS,
                    Manifest.permission.READ_SMS,
                    Manifest.permission.READ_CONTACTS,
                    Manifest.permission.POST_NOTIFICATIONS
                ),
                101
            )
            // Bildirim Dinleme İzni Ekranını Aç
            startActivity(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS))
        }

        @JavascriptInterface
        fun requestModeBDefaultSmsRole() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val roleManager = getSystemService(RoleManager::class.java)
                if (roleManager != null && roleManager.isRoleAvailable(RoleManager.ROLE_SMS)) {
                    val intent = roleManager.createRequestRoleIntent(RoleManager.ROLE_SMS)
                    startActivityForResult(intent, 102)
                }
            }
        }

        @JavascriptInterface
        fun requestCallScreeningRole() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val roleManager = getSystemService(RoleManager::class.java)
                if (roleManager != null && roleManager.isRoleAvailable(RoleManager.ROLE_CALL_SCREENING)) {
                    val intent = roleManager.createRequestRoleIntent(RoleManager.ROLE_CALL_SCREENING)
                    startActivityForResult(intent, 103)
                }
            }
        }
    }
}

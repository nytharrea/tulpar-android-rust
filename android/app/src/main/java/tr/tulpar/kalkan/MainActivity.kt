package tr.tulpar.kalkan

import android.Manifest
import android.annotation.SuppressLint
import android.app.role.RoleManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.webkit.JavascriptInterface
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import org.json.JSONObject

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView

    private val roleRequestLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { _ ->
        notifyWebPermissionUpdate()
    }

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Sistem çubuklarının rengini ve ikon görünürlüğünü ayarla (üst üste binmeyi önler)
        window.statusBarColor = Color.parseColor("#0B1120")
        window.navigationBarColor = Color.parseColor("#0F172A")
        WindowCompat.getInsetsController(window, window.decorView).apply {
            isAppearanceLightStatusBars = false
            isAppearanceLightNavigationBars = false
        }

        RustBridge.initEngine(filesDir.absolutePath)

        webView = WebView(this).apply {
            setBackgroundColor(Color.parseColor("#0B1120"))
            isVerticalScrollBarEnabled = false
            isHorizontalScrollBarEnabled = false
            overScrollMode = WebView.OVER_SCROLL_NEVER
        }
        setContentView(webView)

        // Android 15/16/17 Edge-to-Edge sistem çubuğu boşluklarını otomatik uygula
        ViewCompat.setOnApplyWindowInsetsListener(webView) { view, windowInsets ->
            val insets = windowInsets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            view.setPadding(insets.left, insets.top, insets.right, insets.bottom)
            WindowInsetsCompat.CONSUMED
        }

        webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            cacheMode = WebSettings.LOAD_NO_CACHE
            allowFileAccess = true
            textZoom = 100
        }

        webView.addJavascriptInterface(TulparJsBridge(this), "TulparNative")
        webView.loadUrl("file:///android_asset/index.html")

        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                webView.evaluateJavascript(
                    "(function(){ return window.handleAndroidBack ? window.handleAndroidBack() : false; })()"
                ) { result ->
                    if (result != "true") {
                        isEnabled = false
                        onBackPressedDispatcher.onBackPressed()
                    }
                }
            }
        })
    }

    override fun onResume() {
        super.onResume()
        notifyWebPermissionUpdate()
    }

    private fun notifyWebPermissionUpdate() {
        if (::webView.isInitialized) {
            webView.post {
                webView.evaluateJavascript("window.onNativeResume && window.onNativeResume();", null)
            }
        }
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
        fun getPermissionStatus(): String {
            val notifListeners = Settings.Secure.getString(
                contentResolver,
                "enabled_notification_listeners"
            ) ?: ""
            val hasNotifListener = notifListeners.contains(packageName)
            val hasSmsPerm = ContextCompat.checkSelfPermission(
                this@MainActivity,
                Manifest.permission.RECEIVE_SMS
            ) == PackageManager.PERMISSION_GRANTED

            var hasSmsRole = false
            var hasCallRole = false
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val rm = getSystemService(RoleManager::class.java)
                if (rm != null) {
                    hasSmsRole = rm.isRoleHeld(RoleManager.ROLE_SMS)
                    hasCallRole = rm.isRoleHeld(RoleManager.ROLE_CALL_SCREENING)
                }
            }

            val obj = JSONObject()
            obj.put("notif_listener", hasNotifListener)
            obj.put("sms_perm", hasSmsPerm)
            obj.put("sms_role", hasSmsRole)
            obj.put("call_role", hasCallRole)
            return obj.toString()
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
            val notifListeners = Settings.Secure.getString(
                contentResolver,
                "enabled_notification_listeners"
            ) ?: ""
            if (!notifListeners.contains(packageName)) {
                startActivity(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS))
            }
        }

        @JavascriptInterface
        fun requestModeBDefaultSmsRole() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val roleManager = getSystemService(RoleManager::class.java)
                if (roleManager != null && roleManager.isRoleAvailable(RoleManager.ROLE_SMS)) {
                    val intent = roleManager.createRequestRoleIntent(RoleManager.ROLE_SMS)
                    roleRequestLauncher.launch(intent)
                }
            }
        }

        @JavascriptInterface
        fun requestCallScreeningRole() {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val roleManager = getSystemService(RoleManager::class.java)
                if (roleManager != null && roleManager.isRoleAvailable(RoleManager.ROLE_CALL_SCREENING)) {
                    val intent = roleManager.createRequestRoleIntent(RoleManager.ROLE_CALL_SCREENING)
                    roleRequestLauncher.launch(intent)
                }
            }
        }
    }
}

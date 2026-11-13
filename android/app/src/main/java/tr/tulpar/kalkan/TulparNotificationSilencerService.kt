package tr.tulpar.kalkan

import android.app.Notification
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import org.json.JSONObject

/**
 * YOL A: Akıllı Bildirim Susturucu
 * Kullanıcı Google Mesajlar veya Samsung Mesajlar kullanmaya devam ederken, gelen bildirim
 * spam ise ses ve ekran bildirimini milisaniyeler içinde `cancelNotification(sbn.key)` ile yok eder.
 */
class TulparNotificationSilencerService : NotificationListenerService() {

    private val smsPackages = setOf(
        "com.google.android.apps.messaging",
        "com.samsung.android.messaging",
        "com.android.mms",
        "com.miui.mms"
    )

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        if (sbn == null) return
        val pkg = sbn.packageName ?: return

        // Yalnızca SMS uygulamalarından gelen bildirimleri denetle
        if (!smsPackages.contains(pkg) && !pkg.contains("mms") && !pkg.contains("messaging")) {
            return
        }

        val extras = sbn.notification?.extras ?: return
        val sender = extras.getCharSequence(Notification.EXTRA_TITLE)?.toString() ?: ""
        val body = extras.getCharSequence(Notification.EXTRA_TEXT)?.toString()
            ?: extras.getCharSequence(Notification.EXTRA_BIG_TEXT)?.toString()
            ?: ""

        if (sender.isEmpty() && body.isEmpty()) return

        // 1. SmsDeliverReceiver tarafından milisaniyeler önce spam olarak işaretlendiyse anında sil
        val cachedTime = SmsDeliverReceiver.recentSpamSenders[sender]
        if (cachedTime != null && (System.currentTimeMillis() - cachedTime) < 15_000L) {
            cancelNotification(sbn.key)
            return
        }

        // 2. RCS / Sohbet mesajları için doğrudan Rust motoruna sor
        RustBridge.initEngine(applicationContext.filesDir.absolutePath)
        val verdictJson = RustBridge.inspectSmsJson(sender, body, false)
        val verdict = JSONObject(verdictJson)

        if (verdict.optBoolean("is_spam", false) &&
            verdict.optString("action") == "BLOCK_AND_SILENCE"
        ) {
            cancelNotification(sbn.key)
        }
    }
}

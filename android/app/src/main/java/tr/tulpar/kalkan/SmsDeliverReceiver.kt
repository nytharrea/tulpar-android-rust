package tr.tulpar.kalkan

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.provider.Telephony
import org.json.JSONObject
import java.util.concurrent.ConcurrentHashMap

/**
 * Hem Yol A (RECEIVE_SMS arka plan yakalayıcı) hem de Yol B (SMS_DELIVER varsayılan SMS yöneticisi)
 * için gelen mesajı milisaniyeler içinde Rust motoruna (`RustBridge.inspectSmsJson`) iletir.
 */
class SmsDeliverReceiver : BroadcastReceiver() {
    companion object {
        // Yol A'da NotificationListenerService'in anında bildirim silebilmesi için son spam göndericileri tutar
        val recentSpamSenders = ConcurrentHashMap<String, Long>()
    }

    override fun onReceive(context: Context, intent: Intent) {
        val messages = Telephony.Sms.Intents.getMessagesFromIntent(intent)
        if (messages.isNullOrEmpty()) return

        val sender = messages[0].displayOriginatingAddress ?: messages[0].originatingAddress ?: "Bilinmeyen"
        val fullBody = messages.joinToString(separator = "") { it.displayMessageBody ?: "" }

        // Rust motorunu başlat ve mesajı denetle
        RustBridge.initEngine(context.filesDir.absolutePath)
        val verdictJson = RustBridge.inspectSmsJson(sender, fullBody, true)
        val verdict = JSONObject(verdictJson)

        val isSpam = verdict.optBoolean("is_spam", false)
        if (isSpam) {
            // Yol A için göndericiyi işaretle (NotificationListenerService milisaniyeler içinde bildirimi yok eder)
            recentSpamSenders[sender] = System.currentTimeMillis()
            abortBroadcast() // Eski/uyumlu ROM'larda yayını durdur
        }
    }
}

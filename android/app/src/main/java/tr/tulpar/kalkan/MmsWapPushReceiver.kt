package tr.tulpar.kalkan

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/**
 * Android'in bir uygulamayı Varsayılan SMS Uygulaması (ROLE_SMS - Yol B) olarak
 * kabul etmesi için zorunlu tuttuğu MMS BroadcastReceiver bileşeni.
 */
class MmsWapPushReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        // Yol B MMS alım kancası
    }
}

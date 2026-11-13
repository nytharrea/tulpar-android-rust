package tr.tulpar.kalkan

import android.app.Service
import android.content.Intent
import android.os.IBinder

/**
 * Android'in bir uygulamayı Varsayılan SMS Uygulaması (ROLE_SMS - Yol B) olarak
 * kabul etmesi için zorunlu tuttuğu Hızlı Yanıt Servisi.
 */
class HeadlessSmsSendService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null
}

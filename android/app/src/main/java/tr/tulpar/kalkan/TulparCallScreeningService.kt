package tr.tulpar.kalkan

import android.Manifest
import android.content.pm.PackageManager
import android.net.Uri
import android.provider.ContactsContract
import android.telecom.Call
import android.telecom.CallScreeningService
import androidx.core.content.ContextCompat
import org.json.JSONObject

/**
 * ARAMALAR SEKMESİ (CallScreeningService):
 * Varsayılan telefon uygulamasını değiştirmeden spam numaraları, 0850 pazarlama aramalarını
 * veya gizli numaraları Rust motoru kararıyla doğrudan reddeder ya da sessize alır.
 */
class TulparCallScreeningService : CallScreeningService() {

    override fun onScreenCall(callDetails: Call.Details) {
        if (callDetails.callDirection != Call.Details.DIRECTION_INCOMING) {
            respondToCall(callDetails, CallResponse.Builder().build())
            return
        }

        val handleUri: Uri? = callDetails.handle
        val phoneNumber = handleUri?.schemeSpecificPart ?: "PRIVATE"
        val inContacts = isNumberInContacts(phoneNumber)

        RustBridge.initEngine(applicationContext.filesDir.absolutePath)
        val verdictJson = RustBridge.inspectCallJson(phoneNumber, inContacts)
        val verdict = JSONObject(verdictJson)

        val disallow = verdict.optBoolean("disallow_call", false)
        val reject = verdict.optBoolean("reject_call", false)
        val silence = verdict.optBoolean("silence_call", false)
        val skipNotif = verdict.optBoolean("skip_notification", false)

        val responseBuilder = CallResponse.Builder()
            .setDisallowCall(disallow)
            .setRejectCall(reject)
            .setSilenceCall(silence)
            .setSkipNotification(skipNotif)
            .setSkipCallLog(false)

        respondToCall(callDetails, responseBuilder.build())
    }

    private fun isNumberInContacts(phoneNumber: String): Boolean {
        if (phoneNumber.isBlank() || phoneNumber == "PRIVATE") return false
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_CONTACTS)
            != PackageManager.PERMISSION_GRANTED
        ) {
            return false
        }
        val uri = Uri.withAppendedPath(
            ContactsContract.PhoneLookup.CONTENT_FILTER_URI,
            Uri.encode(phoneNumber)
        )
        contentResolver.query(
            uri,
            arrayOf(ContactsContract.PhoneLookup._ID),
            null,
            null,
            null
        )?.use { cursor ->
            return cursor.moveToFirst()
        }
        return false
    }
}

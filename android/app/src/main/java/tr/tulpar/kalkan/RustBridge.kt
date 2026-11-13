package tr.tulpar.kalkan

object RustBridge {
    init {
        System.loadLibrary("tulpar_core")
    }

    external fun initEngine(storagePath: String): String
    external fun getStateJson(): String
    external fun updateConfigJson(newConfigJson: String): String
    external fun inspectSmsJson(sender: String, body: String, recordMessage: Boolean): String
    external fun inspectCallJson(phoneNumber: String, isInContacts: Boolean): String
    external fun syncUsomNow(): String
}

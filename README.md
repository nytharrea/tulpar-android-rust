# 🐎 Tulpar (𐱃𐰆𐰞𐰯𐰀𐰺) — Türk Mitolojisi Temalı %100 Rust Android SMS & Çağrı Kalkanı

**Tulpar**, Türk mitolojisinde savaşçıları ve yolcuları görünmez tuzaklardan koruyan kanatlı at figüründen ilham alınarak geliştirilmiş, çekirdek filtreleme motoru **%100 Rust (`tulpar_core`)** ile yazılmış modern bir Android SMS ve Çağrı spam engelleme uygulamasıdır.

İOS'teki *Spamstein* tasarımının sadeliğini Android'in güçlü sistem API'leriyle (**Yol A**, **Yol B** ve **CallScreeningService**) birleştirir ve **GitHub Actions** üzerinde otomatik olarak derlenip **V2/V3 imza şemasıyla** imzalanarak doğrudan GitHub Releases üzerinden dağıtılır.

---

## ✨ Temel Özellikler

1. **Çift Çalışma Modu (Kullanıcı Seçimli — Yol A & Yol B):**
   - **Yol A (Akıllı Bildirim Susturucu + Arka Plan Dinleyici — *Varsayılan & En Pratik*):** Mevcut SMS uygulamanızı (Google Mesajlar, Samsung Mesajlar vb.) değiştirmeden kullanmaya devam edersiniz. `RECEIVE_SMS` izniyle gelen mesaj anında Rust motoruna (`Java_tr_tulpar_kalkan_RustBridge_inspectSmsJson`) iletilir; spam ise `NotificationListenerService` devreye girerek bildirim sesini ve ekran bildirimini milisaniyeler içinde yok eder.
   - **Yol B (Tam Varsayılan SMS Uygulaması — `ROLE_SMS`):** Tulpar'ı Android'in varsayılan SMS uygulaması olarak atayabilirsiniz. Tüm SMS/MMS alım bileşenlerini (`SmsDeliverReceiver`, `MmsWapPushReceiver`, `HeadlessSmsSendService`) içerir ve mesajları kendi arayüzündeki **Gelen Kutusu** ile **İstenmeyen** klasörlerine ayırır.
2. **Aramalar Sekmesi (`CallScreeningService`):**
   - Varsayılan telefon uygulamanızı değiştirmeden gizli numaraları, `+90` dışı yabancı spam çağrıları ve `0850` / `02129` gibi pazarlama öneklerini telefonunuz çalmadan reddeder veya sessize alır. Rehberinizdeki kişilere dokunmaz.
3. **T.C. Siber Güvenlik Başkanlığı (USOM) 2026 Entegrasyonu:**
   - USOM'un 1 Haziran 2026'da devreye aldığı yeni JSON REST API (`https://siberguvenlik.gov.tr/api/address/index`) ve güncel 365 günlük aktif zararlı domain beslemesi (`https://connector.klcnetwork.com/feeds/domain-365d.txt`) ile **60.000+ oltalama ve yasa dışı bahis sitesini** çevrimdışı önbelleğe indirir.
4. **Leetspeak Çözücü & Kısaltılmış Link Analizi:**
   - `deneme b0nusu`, `fr33spin`, `c4sino`, `b@his` gibi harf-rakam gizlemelerini Rust katmanında normalize ederek yakalar.
   - `bit.ly`, `tinyurl.com`, `t.ly`, `cutt.ly` gibi kısaltılmış linkleri tespit eder.
5. **Akıllı İzin Listesi (Sahte Link Korumalı):**
   - `doğrulama kodu`, `tek kullanımlık`, `şifreniz` gibi OTP ifadelerini içeren banka/kargo mesajlarını asla engellemez.
   - Alt alan adı desteğine sahiptir (`ty.gl` eklendiğinde `text.ty.gl` otomatik izinlidir). Ancak mesajda izinli bir sitenin yanında ikinci bir sahte link varsa izni iptal ederek adres taklidini önler.

---

## 📂 Proje Mimarisi

```text
tulpar-android-rust/
├── .github/workflows/
│   └── android-release.yml      # Otomatik Rust NDK derleme + APK V2/V3 imzalama + GitHub Release
├── rust-core/                   # %100 Rust Filtreleme Motoru ve JNI Köprüsü
│   ├── Cargo.toml
│   └── src/lib.rs               # USOM senkronizasyonu, Leetspeak çözücü, SMS/Arama karar motoru
└── android/                     # Android Kabuğu (Yol A + Yol B + CallScreeningService)
    ├── app/src/main/
    │   ├── AndroidManifest.xml  # RECEIVE_SMS, ROLE_SMS, NotificationListener, CallScreening izinleri
    │   ├── assets/index.html    # 5 sekmeli sade modern arayüz (Kelimeler, İzinli, Aramalar, Dene, Kurulum)
    │   └── java/tr/tulpar/kalkan/
    │       ├── RustBridge.kt
    │       ├── MainActivity.kt
    │       ├── SmsDeliverReceiver.kt
    │       ├── TulparNotificationSilencerService.kt
    │       ├── TulparCallScreeningService.kt
    │       ├── MmsWapPushReceiver.kt
    │       └── HeadlessSmsSendService.kt
```

---

## 🚀 GitHub Actions ile Derleme ve İmzalama (Sıfır Kurulum)

1. Bu proje dosyalarını yeni bir GitHub deposuna (repository) yükleyin.
2. GitHub deponuzda **Actions** sekmesine gidin; `Tulpar Full-Rust Android APK Build & Sign` iş akışı otomatik olarak çalışacaktır (veya **Run workflow** butonuna basarak tetikleyebilirsiniz).
3. **İmza Anahtarı (Keystore):**
   - Hiçbir GitHub Secret tanımlamasanız bile iş akışı otomatik olarak 2048-bit RSA üretim anahtarı oluşturur, `zipalign` uygular ve Android 14/15+ (Target SDK 35) için zorunlu olan **APK Signature Scheme v2 ve v3** ile imzalar.
   - Kendi kalıcı anahtarınızı kullanmak isterseniz depo ayarlarından (`Settings -> Secrets and variables -> Actions`) şu değişkenleri ekleyebilirsiniz:
     - `SIGNING_KEYSTORE_BASE64`
     - `SIGNING_KEYSTORE_PASS`
     - `SIGNING_KEY_ALIAS`
     - `SIGNING_KEY_PASS`

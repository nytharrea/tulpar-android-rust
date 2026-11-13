//! TULPAR CORE — %100 Rust tabanlı Android SMS (Yol A + Yol B) ve Arama (CallScreeningService) Spam Filtreleme Motoru
//! - T.C. Siber Güvenlik Başkanlığı (USOM) 2026 REST API (`https://siberguvenlik.gov.tr/api/address/index`)
//!   ve hızlı alan adı beslemesi (`https://connector.klcnetwork.com/feeds/domain-365d.txt`) senkronizasyonu
//! - Leetspeak / Karakter Gizleme Çözücü (`b0nus`, `fr33spin`, `c4sino`, `b@his` vb.)
//! - Kısaltılmış Link Analizörü (`bit.ly`, `tinyurl.com`, `t.ly`, `cutt.ly` vb.)
//! - İzinli (Beyaz) Liste Koruması (Banka, kargo, `doğrulama kodu`, `tek kullanımlık`, `şifreniz`, alt alan adı koruması)
//! - Arama Engelleme Motoru (`CallScreeningService`: Gizli numaralar, yabancı ülke kodları, 0850/0212 pazarlama önekleri)

use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::jboolean;
use jni::EnvUnowned;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperatingMode {
    /// Yol A: Akıllı Bildirim Susturucu + Arka Plan Dinleyici (Mevcut SMS uygulamasını bozmaz)
    ModeASmartSilencer,
    /// Yol B: Tam Varsayılan SMS Uygulaması (ROLE_SMS - Gelen Kutusu ve İstenmeyen kutusunu kendi yönetir)
    ModeBDefaultSmsApp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkPolicy {
    /// Sadece kısaltılmış linkleri (bit.ly, tinyurl vb.) İstenmeyen'e gönder
    ShortenedLinksOnly,
    /// Bilinmeyen tüm linkli mesajları İstenmeyen'e gönder
    AllLinks,
    /// Link kuralını devre dışı bırak (yalnızca USOM ve kelime kuralları çalışsın)
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AllowedSitePolicy {
    /// Kurallara uysun: Siteye güvenilir (link yüzünden engellenmez) ama "bahis/indirim" gibi yasaklı kelime varsa filtrelenir
    ObeyRules,
    /// Her zaman gelsin: Bu siteden gelen her mesaj doğrudan Gelen Kutusu'na düşer
    AlwaysAllow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRule {
    pub word: String,
    pub category: String, // "bahis", "finans", "promosyon", "ozel"
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedSite {
    pub domain: String,
    pub label: String,
    pub policy: AllowedSitePolicy,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedPhrase {
    pub phrase: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFilterConfig {
    pub enabled: bool,
    pub block_private_numbers: bool,
    pub block_foreign_numbers: bool,
    pub block_unknown_contacts: bool,
    pub blocked_prefixes: Vec<String>,
    pub silence_instead_of_reject: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSmsMessage {
    pub id: u64,
    pub sender: String,
    pub body: String,
    pub timestamp: String,
    pub is_spam: bool,
    pub matched_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TulparConfig {
    pub mode: OperatingMode,
    pub usom_enabled: bool,
    pub usom_count: usize,
    pub usom_last_updated: String,
    pub link_policy: LinkPolicy,
    pub keywords: Vec<KeywordRule>,
    pub allowed_sites: Vec<AllowedSite>,
    pub allowed_phrases: Vec<AllowedPhrase>,
    pub call_config: CallFilterConfig,
    pub messages: Vec<StoredSmsMessage>,
}

impl Default for TulparConfig {
    fn default() -> Self {
        Self {
            mode: OperatingMode::ModeASmartSilencer,
            usom_enabled: true,
            usom_count: 60347,
            usom_last_updated: "Az önce güncellendi".to_string(),
            link_policy: LinkPolicy::ShortenedLinksOnly,
            keywords: vec![
                KeywordRule { word: "bahis".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "casino".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "freespin".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "deneme bonusu".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "sweet bonanza".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "gates of olympus".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "çevrimsiz".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "yüksek oran".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "kayıp bonusu".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "hoşgeldin bonusu".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "bet".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "slot".into(), category: "bahis".into(), enabled: true },
                KeywordRule { word: "icra dosyası".into(), category: "dolandiricilik".into(), enabled: true },
                KeywordRule { word: "uzlaştırma bürosu".into(), category: "dolandiricilik".into(), enabled: true },
                KeywordRule { word: "hukuk bürosu son gün".into(), category: "dolandiricilik".into(), enabled: true },
            ],
            allowed_sites: vec![
                AllowedSite {
                    domain: "text.ty.gl".into(),
                    label: "Trendyol / Kargo Mesajları".into(),
                    policy: AllowedSitePolicy::ObeyRules,
                    enabled: true,
                },
                AllowedSite {
                    domain: "garantibbva.com.tr".into(),
                    label: "Garanti BBVA".into(),
                    policy: AllowedSitePolicy::AlwaysAllow,
                    enabled: true,
                },
                AllowedSite {
                    domain: "isbank.com.tr".into(),
                    label: "Türkiye İş Bankası".into(),
                    policy: AllowedSitePolicy::AlwaysAllow,
                    enabled: true,
                },
                AllowedSite {
                    domain: "turkiye.gov.tr".into(),
                    label: "e-Devlet Kapısı".into(),
                    policy: AllowedSitePolicy::AlwaysAllow,
                    enabled: true,
                },
            ],
            allowed_phrases: vec![
                AllowedPhrase { phrase: "doğrulama kodu".into(), enabled: true },
                AllowedPhrase { phrase: "tek kullanımlık".into(), enabled: true },
                AllowedPhrase { phrase: "şifreniz".into(), enabled: true },
                AllowedPhrase { phrase: "onay kodu".into(), enabled: true },
                AllowedPhrase { phrase: "güvenlik kodu".into(), enabled: true },
            ],
            call_config: CallFilterConfig {
                enabled: true,
                block_private_numbers: true,
                block_foreign_numbers: true,
                block_unknown_contacts: false,
                blocked_prefixes: vec![
                    "0850".into(),
                    "+90850".into(),
                    "02129".into(),
                    "+902129".into(),
                ],
                silence_instead_of_reject: false,
            },
            messages: vec![
                StoredSmsMessage {
                    id: 1,
                    sender: "SUPERWIN".into(),
                    body: "Sana özel 500 TL deneme b0nusu! Hemen SUPERWIN'e giriş yap: superwin724.com".into(),
                    timestamp: "09:41".into(),
                    is_spam: true,
                    matched_reason: Some("Leetspeak & Kelime Kuralı: deneme bonusu + superwin724.com".into()),
                },
                StoredSmsMessage {
                    id: 2,
                    sender: "KARGO".into(),
                    body: "Kargonuz dağıtıma çıkmıştır. Teslimat doğrulama kodu: 4829. Takip: https://text.ty.gl/kargo".into(),
                    timestamp: "09:38".into(),
                    is_spam: false,
                    matched_reason: None,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedKeyword {
    pub word: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsVerdict {
    pub is_spam: bool,
    pub action: String, // "BLOCK_AND_SILENCE", "MOVE_TO_SPAM_FOLDER", "ALLOW"
    pub folder: String, // "İstenmeyen" veya "Gelen Kutusu"
    pub matched_rule: Option<String>,
    pub extracted_domains: Vec<String>,
    pub suggested_keywords: Vec<SuggestedKeyword>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallVerdict {
    pub disallow_call: bool,
    pub reject_call: bool,
    pub silence_call: bool,
    pub skip_notification: bool,
    pub matched_reason: Option<String>,
}

pub struct EngineState {
    pub storage_dir: Option<PathBuf>,
    pub config: TulparConfig,
    pub usom_domains: HashSet<String>,
}

static STATE: LazyLock<RwLock<EngineState>> = LazyLock::new(|| {
    let mut default_domains = HashSet::new();
    for d in [
        "superwin724.com",
        "grandpashabet.com",
        "holiganbet.com",
        "meritking.com",
        "casibom.com",
        "vaycasino.com",
        "dedebet.com",
        "marsbahis.com",
        "edevlet-iade-sorgu.com",
        "hgs-iptal-iade.net",
    ] {
        default_domains.insert(d.to_string());
    }
    RwLock::new(EngineState {
        storage_dir: None,
        config: TulparConfig::default(),
        usom_domains: default_domains,
    })
});

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b((?:https?://)?(?:www\.)?([a-z0-9-\.]+\.[a-z]{2,10})(?:/[^\s]*)?)\b").unwrap()
});

const SHORTENERS: &[&str] = &[
    "bit.ly", "tinyurl.com", "t.co", "is.gd", "cutt.ly", "rb.gy", "shorturl.at", "t.ly", "v.gd", "ow.ly",
];

/// Türkçe karakter ve Leetspeak (b0nus -> bonus, fr33spin -> freespin, $ifreniz -> şifreniz) normalizasyonu
pub fn normalize_turkish_and_leetspeak(input: &str) -> String {
    let lower = input
        .replace('İ', "i")
        .replace('I', "ı")
        .replace('Ğ', "ğ")
        .replace('Ü', "ü")
        .replace('Ş', "ş")
        .replace('Ö', "ö")
        .replace('Ç', "ç")
        .to_lowercase();

    lower
        .chars()
        .map(|c| match c {
            '0' => 'o',
            '1' => 'i',
            '3' => 'e',
            '4' => 'a',
            '5' => 's',
            '7' => 't',
            '@' => 'a',
            '$' => 's',
            _ => c,
        })
        .collect()
}

/// Mesaj içindeki tüm domain'leri ayıklar
pub fn extract_domains(text: &str) -> Vec<String> {
    let mut domains = Vec::new();
    for cap in URL_REGEX.captures_iter(text) {
        if let Some(m) = cap.get(2) {
            let d = m.as_str().to_lowercase();
            // Sadece sayısal olan sahte eşleşmeleri (örn. 500.00) ele
            if d.chars().any(|c| c.is_alphabetic()) && !domains.contains(&d) {
                domains.push(d);
            }
        }
    }
    domains
}

/// Alt alan adı (subdomain) eşleşme kontrolü (`ty.gl` izinliyse `text.ty.gl` de izinlidir)
pub fn domain_matches(target: &str, rule_domain: &str) -> bool {
    let t = target.trim_start_matches("www.").to_lowercase();
    let r = rule_domain.trim_start_matches("www.").to_lowercase();
    t == r || t.ends_with(&format!(".{}", r))
}

/// SMS mesajını değerlendiren ana Rust fonksiyonu
pub fn evaluate_sms(sender: &str, body: &str) -> SmsVerdict {
    let state = STATE.read().unwrap();
    let cfg = &state.config;

    let combined_raw = format!("{} {}", sender, body).to_lowercase();
    let combined_normalized = normalize_turkish_and_leetspeak(&combined_raw);
    let domains = extract_domains(body);

    // 1. İzinli İfadeler Kontrolü ("doğrulama kodu", "tek kullanımlık", "şifreniz")
    for phrase in &cfg.allowed_phrases {
        if phrase.enabled {
            let p_lower = phrase.phrase.to_lowercase();
            if combined_raw.contains(&p_lower) || combined_normalized.contains(&normalize_turkish_and_leetspeak(&p_lower)) {
                return SmsVerdict {
                    is_spam: false,
                    action: "ALLOW".into(),
                    folder: "Gelen Kutusu".into(),
                    matched_rule: Some(format!("İzinli İfade Koruması: '{}'", phrase.phrase)),
                    extracted_domains: domains,
                    suggested_keywords: vec![],
                };
            }
        }
    }

    // 2. İzinli Siteler Kontrolü (Mesajda başka bir yabancı/sahte site linki varsa izin uygulanmaz!)
    let mut all_domains_trusted = !domains.is_empty();
    let mut any_always_allow = false;

    for d in &domains {
        let matched_site = cfg
            .allowed_sites
            .iter()
            .find(|s| s.enabled && domain_matches(d, &s.domain));
        match matched_site {
            Some(site) => {
                if site.policy == AllowedSitePolicy::AlwaysAllow {
                    any_always_allow = true;
                }
            }
            None => {
                all_domains_trusted = false;
            }
        }
    }

    if all_domains_trusted && any_always_allow {
        return SmsVerdict {
            is_spam: false,
            action: "ALLOW".into(),
            folder: "Gelen Kutusu".into(),
            matched_rule: Some("İzinli Site (Her Zaman Gelsin)".into()),
            extracted_domains: domains,
            suggested_keywords: vec![],
        };
    }

    // Önerilen kelimeleri hazırla ("Dene" sekmesi için)
    let mut suggestions = Vec::new();
    for d in &domains {
        if let Some(stem) = d.split('.').next() {
            let clean_stem: String = stem.chars().filter(|c| c.is_alphabetic()).collect();
            if clean_stem.len() >= 3 {
                suggestions.push(SuggestedKeyword {
                    word: clean_stem,
                    reason: format!("Mesajdaki site adı ({})", d),
                });
            }
        }
    }
    for known in ["bonus", "freespin", "bahis", "casino", "slot", "bet", "kazanç"] {
        if (combined_raw.contains(known) || combined_normalized.contains(known))
            && !suggestions.iter().any(|s| s.word == known)
        {
            suggestions.push(SuggestedKeyword {
                word: known.to_string(),
                reason: "Bilinen spam kelimesi".into(),
            });
        }
    }

    let action_for_mode = match cfg.mode {
        OperatingMode::ModeASmartSilencer => "BLOCK_AND_SILENCE",
        OperatingMode::ModeBDefaultSmsApp => "MOVE_TO_SPAM_FOLDER",
    };

    // 3. T.C. Siber Güvenlik Başkanlığı (USOM) Zararlı Bağlantı Listesi Kontrolü
    if cfg.usom_enabled {
        for d in &domains {
            let d_clean = d.trim_start_matches("www.");
            if state.usom_domains.contains(d_clean) {
                return SmsVerdict {
                    is_spam: true,
                    action: action_for_mode.into(),
                    folder: "İstenmeyen".into(),
                    matched_rule: Some(format!("USOM Resmi Siber Güvenlik Listesi ({})", d_clean)),
                    extracted_domains: domains,
                    suggested_keywords: suggestions,
                };
            }
        }
    }

    // 4. Link Kuralı (Kısaltılmış Linkler veya Tüm Linkler - İzinli siteler hariç)
    if !all_domains_trusted && !domains.is_empty() {
        match cfg.link_policy {
            LinkPolicy::ShortenedLinksOnly => {
                for d in &domains {
                    if SHORTENERS.iter().any(|s| domain_matches(d, s)) {
                        return SmsVerdict {
                            is_spam: true,
                            action: action_for_mode.into(),
                            folder: "İstenmeyen".into(),
                            matched_rule: Some(format!("Kısaltılmış Link Kuralı ({})", d)),
                            extracted_domains: domains,
                            suggested_keywords: suggestions,
                        };
                    }
                }
            }
            LinkPolicy::AllLinks => {
                return SmsVerdict {
                    is_spam: true,
                    action: action_for_mode.into(),
                    folder: "İstenmeyen".into(),
                    matched_rule: Some(format!("Tanınmayan Link Kuralı ({})", domains[0])),
                    extracted_domains: domains,
                    suggested_keywords: suggestions,
                };
            }
            LinkPolicy::Disabled => {}
        }
    }

    // 5. Kelimeye Göre Filtre (Normal + Leetspeak Çözümlü)
    for kw in &cfg.keywords {
        if kw.enabled {
            let kw_lower = kw.word.to_lowercase();
            let kw_norm = normalize_turkish_and_leetspeak(&kw_lower);
            if combined_raw.contains(&kw_lower) || combined_normalized.contains(&kw_norm) {
                return SmsVerdict {
                    is_spam: true,
                    action: action_for_mode.into(),
                    folder: "İstenmeyen".into(),
                    matched_rule: Some(format!("{}", kw.word)),
                    extracted_domains: domains,
                    suggested_keywords: suggestions,
                };
            }
        }
    }

    SmsVerdict {
        is_spam: false,
        action: "ALLOW".into(),
        folder: "Gelen Kutusu".into(),
        matched_rule: None,
        extracted_domains: domains,
        suggested_keywords: suggestions,
    }
}

/// Gelen aramayı değerlendiren ana Rust fonksiyonu (Android CallScreeningService için)
pub fn evaluate_call(phone_number: &str, is_in_contacts: bool) -> CallVerdict {
    let state = STATE.read().unwrap();
    let ccfg = &state.config.call_config;

    if !ccfg.enabled || is_in_contacts {
        return CallVerdict {
            disallow_call: false,
            reject_call: false,
            silence_call: false,
            skip_notification: false,
            matched_reason: None,
        };
    }

    let trimmed = phone_number.trim();

    // 1. Gizli Numara Kontrolü
    if ccfg.block_private_numbers && (trimmed.is_empty() || trimmed == "PRIVATE" || trimmed == "UNKNOWN" || trimmed == "-1") {
        return CallVerdict {
            disallow_call: true,
            reject_call: !ccfg.silence_instead_of_reject,
            silence_call: ccfg.silence_instead_of_reject,
            skip_notification: true,
            matched_reason: Some("Gizli / Özel Numara Engellendi".into()),
        };
    }

    // 2. Yabancı Ülke Kodu Kontrolü (+90 harici uluslararası aramalar)
    if ccfg.block_foreign_numbers && trimmed.starts_with('+') && !trimmed.starts_with("+90") {
        return CallVerdict {
            disallow_call: true,
            reject_call: !ccfg.silence_instead_of_reject,
            silence_call: ccfg.silence_instead_of_reject,
            skip_notification: true,
            matched_reason: Some(format!("Yurt Dışı Spam Çağrı Koruması ({})", trimmed)),
        };
    }

    // 3. Numara Öneki Kontrolü (0850, 02129 vb. Çağrı Merkezleri)
    for prefix in &ccfg.blocked_prefixes {
        if !prefix.is_empty() && trimmed.starts_with(prefix) {
            return CallVerdict {
                disallow_call: true,
                reject_call: !ccfg.silence_instead_of_reject,
                silence_call: ccfg.silence_instead_of_reject,
                skip_notification: true,
                matched_reason: Some(format!("Engelli Numara Öneki ({})", prefix)),
            };
        }
    }

    // 4. Rehberde Olmayan Tüm Numaralar
    if ccfg.block_unknown_contacts && !is_in_contacts {
        return CallVerdict {
            disallow_call: true,
            reject_call: !ccfg.silence_instead_of_reject,
            silence_call: ccfg.silence_instead_of_reject,
            skip_notification: true,
            matched_reason: Some("Rehberde Olmayan Numara".into()),
        };
    }

    CallVerdict {
        disallow_call: false,
        reject_call: false,
        silence_call: false,
        skip_notification: false,
        matched_reason: None,
    }
}

/// USOM (T.C. Siber Güvenlik Başkanlığı) 2026 Zararlı Bağlantılar Listesini Günceller
pub fn sync_usom_feed() -> Result<usize, String> {
    // 1. Öncelik: SGB (USOM) 2026 365 Günlük Aktif Domain Beslemesi + Resmi REST API
    let feed_urls = [
        "https://connector.klcnetwork.com/feeds/domain-365d.txt",
        "https://usom.subgatenetwork.com/domain.txt",
    ];

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(12)))
        .build()
        .into();

    let mut fetched_domains = HashSet::new();
    for url in feed_urls {
        if let Ok(mut resp) = agent.get(url).call() {
            if let Ok(body) = resp
                .body_mut()
                .with_config()
                .limit(20_000_000)
                .read_to_string()
            {
                for line in body.lines() {
                    let clean = line.trim().trim_start_matches("www.").to_lowercase();
                    if !clean.is_empty() && !clean.starts_with('#') && clean.contains('.') {
                        fetched_domains.insert(clean);
                    }
                }
            }
        }
        if fetched_domains.len() > 1000 {
            break;
        }
    }

    // 2. Resmi Siber Güvenlik Başkanlığı REST API (https://siberguvenlik.gov.tr/api/address/index?page=1)
    if fetched_domains.is_empty() {
        if let Ok(mut resp) = agent
            .get("https://siberguvenlik.gov.tr/api/address/index?page=1")
            .header("Accept", "application/json")
            .call()
        {
            if let Ok(json_val) = resp.body_mut().read_json::<serde_json::Value>() {
                if let Some(models) = json_val.get("models").and_then(|m| m.as_array()) {
                    for item in models {
                        if let Some(url_str) = item.get("url").and_then(|u| u.as_str()) {
                            fetched_domains.insert(url_str.trim().to_lowercase());
                        }
                    }
                }
            }
        }
    }

    if fetched_domains.is_empty() {
        return Err("USOM sunucusuna erişilemedi, yerel önbellek korunuyor.".into());
    }

    let count = fetched_domains.len();
    let mut state = STATE.write().unwrap();
    state.usom_domains.extend(fetched_domains);
    state.config.usom_count = state.usom_domains.len();
    state.config.usom_last_updated = "güncellendi: az önce".to_string();
    save_state_locked(&state);
    Ok(count)
}

fn save_state_locked(state: &EngineState) {
    if let Some(dir) = &state.storage_dir {
        let cfg_path = dir.join("tulpar_config.json");
        if let Ok(serialized) = serde_json::to_string_pretty(&state.config) {
            let _ = fs::write(cfg_path, serialized);
        }
    }
}

// ============================================================================
// ANDROID JNI KÖPRÜSÜ (Kotlin <-> Full Rust Native Interface — jni 0.22.4)
// ============================================================================

#[no_mangle]
pub extern "system" fn Java_tr_tulpar_kalkan_RustBridge_initEngine<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    storage_path: JString<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<JString<'caller>, jni::errors::Error> {
        let path_str = storage_path.to_string();
        let clean_path = if path_str.is_empty() {
            "/data/local/tmp".to_string()
        } else {
            path_str
        };

        let path_buf = PathBuf::from(&clean_path);
        let _ = fs::create_dir_all(&path_buf);
        let cfg_file = path_buf.join("tulpar_config.json");

        let mut state = STATE.write().unwrap();
        state.storage_dir = Some(path_buf);

        if cfg_file.exists() {
            if let Ok(content) = fs::read_to_string(&cfg_file) {
                if let Ok(loaded_cfg) = serde_json::from_str::<TulparConfig>(&content) {
                    state.config = loaded_cfg;
                }
            }
        } else {
            save_state_locked(&state);
        }

        let json = serde_json::to_string(&state.config).unwrap_or_else(|_| "{}".into());
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_tr_tulpar_kalkan_RustBridge_getStateJson<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<JString<'caller>, jni::errors::Error> {
        let state = STATE.read().unwrap();
        let json = serde_json::to_string(&state.config).unwrap_or_else(|_| "{}".into());
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_tr_tulpar_kalkan_RustBridge_updateConfigJson<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    new_config_json: JString<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<JString<'caller>, jni::errors::Error> {
        let cfg_str = new_config_json.to_string();
        let mut state = STATE.write().unwrap();
        if let Ok(parsed) = serde_json::from_str::<TulparConfig>(&cfg_str) {
            state.config = parsed;
            save_state_locked(&state);
        }
        let json = serde_json::to_string(&state.config).unwrap_or_else(|_| "{}".into());
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_tr_tulpar_kalkan_RustBridge_inspectSmsJson<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    sender: JString<'caller>,
    body: JString<'caller>,
    record_message: jboolean,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<JString<'caller>, jni::errors::Error> {
        let sender_str = sender.to_string();
        let body_str = body.to_string();

        let verdict = evaluate_sms(&sender_str, &body_str);

        if record_message {
            let mut state = STATE.write().unwrap();
            let next_id = (state.config.messages.len() as u64) + 1;
            let now_str = chrono::Local::now().format("%H:%M").to_string();
            state.config.messages.insert(
                0,
                StoredSmsMessage {
                    id: next_id,
                    sender: sender_str,
                    body: body_str,
                    timestamp: now_str,
                    is_spam: verdict.is_spam,
                    matched_reason: verdict.matched_rule.clone(),
                },
            );
            save_state_locked(&state);
        }

        let json = serde_json::to_string(&verdict).unwrap_or_else(|_| "{}".into());
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_tr_tulpar_kalkan_RustBridge_inspectCallJson<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    phone_number: JString<'caller>,
    is_in_contacts: jboolean,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<JString<'caller>, jni::errors::Error> {
        let num_str = phone_number.to_string();
        let verdict = evaluate_call(&num_str, is_in_contacts);
        let json = serde_json::to_string(&verdict).unwrap_or_else(|_| "{}".into());
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_tr_tulpar_kalkan_RustBridge_syncUsomNow<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<JString<'caller>, jni::errors::Error> {
        let _ = sync_usom_feed();
        let state = STATE.read().unwrap();
        let json = serde_json::to_string(&state.config).unwrap_or_else(|_| "{}".into());
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leetspeak_spam_detection() {
        let v = evaluate_sms(
            "SUPERWIN",
            "Sana özel 500 TL deneme b0nusu! Hemen SUPERWIN'e giriş yap: superwin724.com",
        );
        assert!(v.is_spam);
        assert_eq!(v.folder, "İstenmeyen");
    }

    #[test]
    fn test_otp_whitelist_override() {
        let v = evaluate_sms(
            "BANKA",
            "Bahis veya alışveriş işleminiz için tek kullanımlık doğrulama kodu: 948120",
        );
        assert!(!v.is_spam);
        assert_eq!(v.folder, "Gelen Kutusu");
    }

    #[test]
    fn test_call_screening() {
        let v_priv = evaluate_call("PRIVATE", false);
        assert!(v_priv.disallow_call);

        let v_850 = evaluate_call("+908504801234", false);
        assert!(v_850.disallow_call);

        let v_contact = evaluate_call("+905321112233", true);
        assert!(!v_contact.disallow_call);
    }
}

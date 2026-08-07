use std::{
    collections::HashMap,
    env, fs,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    keepa_api_key: Option<String>,
    product_cache: Arc<Mutex<HashMap<String, CachedProduct>>>,
    llm_api_key: Option<String>,
    llm_model: String,
    llm_base_url: String,
    client: reqwest::Client,
}

#[derive(Clone)]
struct CachedProduct {
    fetched_at: Instant,
    product: Product,
}

#[derive(Deserialize)]
struct CreativeRequest {
    input: String,
    marketplace: Option<String>,
    language: Language,
    #[serde(default)]
    tone: Tone,
    #[serde(default = "default_goal")]
    goal: String,
}

fn default_goal() -> String {
    "conversion".into()
}

#[derive(Serialize)]
struct CreativeResponse {
    product: Product,
    insights: ProductInsights,
    scripts: Vec<TiktokScript>,
}

#[derive(Serialize, Deserialize)]
struct ScriptBundle {
    scripts: Vec<TiktokScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    asin: String,
    marketplace: String,
    source_url: String,
    title: String,
    main_image_url: String,
    bullet_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductInsights {
    product_summary: String,
    core_selling_points: Vec<SellingPoint>,
    target_segments: Vec<TargetSegment>,
    content_angles: Vec<ContentAngle>,
    claim_boundaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SellingPoint {
    feature: String,
    customer_value: String,
    evidence: String,
    confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TargetSegment {
    segment: String,
    need: String,
    rationale: String,
    confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentAngle {
    name: String,
    insight: String,
    hook_direction: String,
    supporting_features: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Language {
    En,
    Es,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Tone {
    Energetic,
    Ugc,
    Premium,
}
impl Default for Tone {
    fn default() -> Self {
        Self::Ugc
    }
}

#[derive(Serialize, Deserialize)]
struct TiktokScript {
    #[serde(default)]
    title: String,
    #[serde(default)]
    angle: String,
    language: String,
    hook: String,
    voiceover: String,
    scenes: Vec<Scene>,
    caption: String,
    hashtags: Vec<String>,
    disclosure: String,
}
#[derive(Serialize, Deserialize)]
struct Scene {
    seconds: String,
    visual: String,
    narration: String,
    #[serde(default)]
    chinese_translation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    on_screen_text: Option<String>,
}

#[derive(Deserialize)]
struct KeepaResponse {
    products: Option<Vec<KeepaProduct>>,
    error: Option<Value>,
}

#[derive(Deserialize)]
struct KeepaProduct {
    asin: String,
    title: Option<String>,
    features: Option<Vec<String>>,
    images: Option<Vec<KeepaImage>>,
}

#[derive(Deserialize)]
struct KeepaImage {
    #[serde(rename = "l")]
    large: Option<String>,
    #[serde(rename = "m")]
    medium: Option<String>,
}

fn load_keepa_key() -> Option<String> {
    if let Some(key) = env::var("KEEPA_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return Some(key);
    }
    let path = env::var("KEEPA_KEY_FILE").unwrap_or_else(|_| "../keepa_key.json".into());
    let contents = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&contents).ok()?;
    value
        .get("KEEPA_API_KEY")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_string)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let state = AppState {
        keepa_api_key: load_keepa_key(),
        product_cache: Arc::new(Mutex::new(HashMap::new())),
        llm_api_key: env::var("DEEPSEEK_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        llm_model: env::var("LLM_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".into()),
        llm_base_url: env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com".into()),
        client: reqwest::Client::new(),
    };
    let app = Router::new()
        .route("/api/health", axum::routing::get(health))
        .route("/api/creative/generate", post(generate_creative))
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("API listening on http://{bind_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "keepa_configured": state.keepa_api_key.is_some(),
        "deepseek_configured": state.llm_api_key.is_some(),
        "llm_model": state.llm_model,
    }))
}

fn normalize_product_input(
    input: &str,
    fallback_marketplace: Option<&str>,
) -> Result<(String, String), ApiError> {
    let trimmed = input.trim();
    let mut marketplace = fallback_marketplace.unwrap_or("com").to_lowercase();
    let mut asin = trimmed.to_ascii_uppercase();
    let lower = trimmed.to_lowercase();
    if lower.contains("amazon.") {
        if let Some(host_start) = lower.find("amazon.") {
            let suffix = &lower[host_start + "amazon.".len()..];
            let domain = suffix.split('/').next().unwrap_or("com");
            if [
                "com", "co.uk", "de", "es", "fr", "it", "co.jp", "ca", "com.mx",
            ]
            .contains(&domain)
            {
                marketplace = domain.to_string();
            }
        }
        let path_marker = if let Some(index) = lower.find("/dp/") {
            Some((index, 4))
        } else {
            lower.find("/gp/product/").map(|index| (index, 12))
        };
        if let Some((index, marker_len)) = path_marker {
            asin = trimmed[index + marker_len..]
                .chars()
                .take(10)
                .collect::<String>()
                .to_ascii_uppercase();
        }
    }
    if !asin.chars().all(|c| c.is_ascii_alphanumeric()) || asin.len() != 10 {
        return Err(ApiError::bad_request(
            "请输入有效的 Amazon 商品链接或 10 位 ASIN。",
        ));
    }
    Ok((asin, marketplace))
}

async fn fetch_keepa_product(
    state: &AppState,
    asin: &str,
    marketplace: &str,
) -> Result<Product, ApiError> {
    let cache_key = format!("{marketplace}:{asin}");
    if let Ok(cache) = state.product_cache.lock() {
        if let Some(cached) = cache.get(&cache_key) {
            if cached.fetched_at.elapsed() < Duration::from_secs(6 * 60 * 60) {
                return Ok(cached.product.clone());
            }
        }
    }
    let key = state.keepa_api_key.as_ref().ok_or_else(|| {
        ApiError::service_unavailable("尚未配置 Keepa API Key，请检查 keepa_key.json。")
    })?;
    let domain_id = keepa_domain_id(marketplace)
        .ok_or_else(|| ApiError::bad_request("Keepa 暂不支持该 Amazon 站点。"))?;
    let response = state
        .client
        .get("https://api.keepa.com/product")
        .query(&[
            ("key", key.as_str()),
            ("domain", domain_id),
            ("asin", asin),
            ("history", "0"),
            ("stats", "0"),
        ])
        .send()
        .await
        .map_err(|_| ApiError::bad_gateway("无法连接 Keepa，请稍后重试。"))?;
    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(format!(
            "Keepa 请求失败（{}）。",
            response.status().as_u16()
        )));
    }
    let payload: KeepaResponse = response
        .json()
        .await
        .map_err(|_| ApiError::bad_gateway("Keepa 返回了无法解析的数据。"))?;
    if payload.error.is_some() {
        return Err(ApiError::bad_gateway(
            "Keepa 拒绝了商品请求，请检查额度或订阅权限。",
        ));
    }
    let keepa_product = payload
        .products
        .and_then(|mut products| products.drain(..).next())
        .ok_or_else(|| ApiError::bad_gateway("Keepa 没有找到这个 ASIN。"))?;
    let title = keepa_product
        .title
        .map(|value| clean_keepa_text(&value))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::bad_gateway("Keepa 暂无该商品的标题数据。"))?;
    let bullet_points = keepa_product
        .features
        .unwrap_or_default()
        .into_iter()
        .map(|value| clean_keepa_text(&value))
        .filter(|value| !value.is_empty())
        .take(5)
        .collect();
    let image_name = keepa_product
        .images
        .unwrap_or_default()
        .into_iter()
        .find_map(|image| image.large.or(image.medium))
        .unwrap_or_default();
    let main_image_url = if image_name.is_empty() {
        String::new()
    } else {
        format!("https://m.media-amazon.com/images/I/{image_name}")
    };

    let product = Product {
        asin: keepa_product.asin,
        marketplace: marketplace.to_string(),
        source_url: format!("https://www.amazon.{marketplace}/dp/{asin}"),
        title,
        main_image_url,
        bullet_points,
    };
    if let Ok(mut cache) = state.product_cache.lock() {
        cache.insert(
            cache_key,
            CachedProduct {
                fetched_at: Instant::now(),
                product: product.clone(),
            },
        );
    }
    Ok(product)
}

fn keepa_domain_id(marketplace: &str) -> Option<&'static str> {
    match marketplace {
        "com" => Some("1"),
        "co.uk" => Some("2"),
        "de" => Some("3"),
        "fr" => Some("4"),
        "co.jp" => Some("5"),
        "ca" => Some("6"),
        "it" => Some("8"),
        "es" => Some("9"),
        "com.mx" => Some("11"),
        _ => None,
    }
}

fn clean_keepa_text(value: &str) -> String {
    let mut text = String::with_capacity(value.len());
    let mut inside_tag = false;
    for character in value.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

async fn call_llm_json<T: DeserializeOwned>(
    state: &AppState,
    system_prompt: &str,
    prompt: &str,
    temperature: f32,
) -> Result<T, ApiError> {
    let key = state.llm_api_key.as_ref().ok_or_else(|| {
        ApiError::service_unavailable(
            "尚未配置 DEEPSEEK_API_KEY，请在 backend/.env 中配置密钥并重启后端。",
        )
    })?;
    let url = format!(
        "{}/chat/completions",
        state.llm_base_url.trim_end_matches('/')
    );
    let mut last_error = "模型返回了空内容".to_string();
    for attempt in 0..2 {
        let attempt_prompt = if attempt == 0 {
            prompt.to_string()
        } else {
            format!("{prompt}\n\nRETRY: The previous output failed validation: {last_error}. Return one corrected, complete JSON object without markdown fences.")
        };
        let response = state.client.post(&url).bearer_auth(key).json(&json!({
            "model": state.llm_model, "temperature": temperature, "max_tokens": 3600,
            "thinking": { "type": "disabled" },
            "response_format": { "type": "json_object" },
            "messages": [{ "role": "system", "content": system_prompt }, { "role": "user", "content": attempt_prompt }]
        })).send().await.map_err(|error| ApiError::bad_gateway(format!("产品分析模型请求失败：{error}")))?
        .error_for_status().map_err(|error| ApiError::bad_gateway(format!("产品分析模型响应失败：{error}")))?;
        let body: Value = response
            .json()
            .await
            .map_err(|error| ApiError::bad_gateway(format!("无法解析产品分析响应：{error}")))?;
        let content = body
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .unwrap_or("");
        if content.trim().is_empty() {
            last_error = "模型返回了空内容".to_string();
            continue;
        }
        match parse_json_content(content) {
            Ok(result) => return Ok(result),
            Err(error) => last_error = format!("JSON 结构错误：{error}"),
        }
    }
    Err(ApiError::bad_gateway(format!(
        "模型连续两次未返回可用结果：{last_error}"
    )))
}

async fn generate_creative(
    State(state): State<AppState>,
    Json(request): Json<CreativeRequest>,
) -> Result<Json<CreativeResponse>, ApiError> {
    let (asin, marketplace) =
        normalize_product_input(&request.input, request.marketplace.as_deref())?;
    let product = fetch_keepa_product(&state, &asin, &marketplace).await?;

    let analysis_prompt = build_analysis_prompt(&product);
    let insights =
        call_llm_json::<ProductInsights>(&state, ANALYSIS_SYSTEM_PROMPT, &analysis_prompt, 0.45)
            .await?;

    let base_prompt = build_bundle_prompt(
        &product,
        &insights,
        request.language,
        request.tone,
        &request.goal,
    );
    let mut prompt = base_prompt.clone();
    for _ in 0..3 {
        let bundle = call_llm_json::<ScriptBundle>(&state, SYSTEM_PROMPT, &prompt, 0.8).await?;
        let validation_error = if bundle.scripts.len() != 3 {
            "没有返回三个完整的创意方向".to_string()
        } else if let Some(error) = bundle
            .scripts
            .iter()
            .find_map(|script| validate_script_claims(script, &product).err())
        {
            error
        } else {
            return Ok(Json(CreativeResponse {
                product,
                insights,
                scripts: bundle.scripts,
            }));
        };
        prompt = format!(
            "{base_prompt}\n\nMANDATORY CORRECTION: The previous scripts failed the factual quality gate because they {validation_error}. Rewrite all three scripts. Replace personal-experience language with neutral, observable product demonstration language. Return only the corrected JSON object."
        );
    }
    Err(ApiError::bad_gateway(
        "脚本没有通过事实校验，请重新生成一组。",
    ))
}

fn validate_script_claims(script: &TiktokScript, product: &Product) -> Result<(), String> {
    let source = format!("{} {}", product.title, product.bullet_points.join(" ")).to_lowercase();
    let output = format!(
        "{} {} {} {} {}",
        script.hook,
        script.voiceover,
        script.caption,
        script.hashtags.join(" "),
        script
            .scenes
            .iter()
            .map(|scene| format!(
                "{} {} {}",
                scene.visual,
                scene.narration,
                scene.on_screen_text.as_deref().unwrap_or("")
            ))
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_lowercase();
    let unsupported_phrases = [
        "i tested",
        "i've tested",
        "i tried",
        "i've tried",
        "i use",
        "i take it",
        "i love",
        "my favorite",
        "life-changing",
        "life changing",
        "guaranteed",
        "viral",
        "the best",
        "lo probé",
        "he probado",
        "lo uso",
        "me encanta",
        "mi favorito",
        "garantizado",
        "viral",
    ];
    for phrase in unsupported_phrases {
        if output.contains(phrase) && !source.contains(phrase) {
            return Err(format!("包含无法由产品资料证明的表述：{phrase}"));
        }
    }
    Ok(())
}

fn parse_json_content<T: DeserializeOwned>(content: &str) -> Result<T, serde_json::Error> {
    let trimmed = content.trim();
    let without_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let without_fence = without_prefix
        .strip_suffix("```")
        .unwrap_or(without_prefix)
        .trim();

    match serde_json::from_str(without_fence) {
        Ok(script) => Ok(script),
        Err(initial_error) => {
            if let (Some(start), Some(end)) = (without_fence.find('{'), without_fence.rfind('}')) {
                if start < end {
                    return serde_json::from_str(&without_fence[start..=end]);
                }
            }
            Err(initial_error)
        }
    }
}

fn build_analysis_prompt(product: &Product) -> String {
    format!(
        r#"Analyze the Amazon title and bullet points below before any video script is written. Separate source facts from marketing inferences. Write the analysis fields in concise Simplified Chinese.

Return ONLY one valid JSON object with exactly this schema:
{{
  "product_summary": "one-sentence factual positioning",
  "core_selling_points": [
    {{"feature":"source-grounded feature", "customer_value":"careful consumer value inference", "evidence":"exact or close paraphrase from source", "confidence":"high|medium|low"}}
  ],
  "target_segments": [
    {{"segment":"possible audience", "need":"relevant need", "rationale":"why the facts may fit", "confidence":"high|medium|low"}}
  ],
  "content_angles": [
    {{"name":"short angle name", "insight":"the consumer tension", "hook_direction":"a hook strategy, not final copy", "supporting_features":["features that substantiate this angle"]}}
  ],
  "claim_boundaries": ["claims the later script must not make"]
}}

Requirements:
- Return 3–5 core selling points, 2–4 possible audience segments, and exactly 3 materially different content angles.
- Evidence must be traceable to the supplied title or bullets. Never treat an inferred benefit, audience, or use case as a product fact.
- Do not invent price, discount, materials, dimensions, ingredients, compatibility, certification, performance, reviews, popularity, safety, or results.
- Add ambiguous, exaggerated, or unsupported claims to claim_boundaries.

Source product JSON:
{}"#,
        serde_json::to_string(product).unwrap()
    )
}

fn build_bundle_prompt(
    product: &Product,
    insights: &ProductInsights,
    language: Language,
    tone: Tone,
    goal: &str,
) -> String {
    let lang = match language {
        Language::En => "English",
        Language::Es => "Spanish (neutral Latin American)",
    };
    let tone = match tone {
        Tone::Energetic => "energetic",
        Tone::Ugc => "natural creator-style",
        Tone::Premium => "clean premium",
    };
    let goal_label = match goal {
        "organic" => "organic engagement and saves",
        "awareness" => "product awareness",
        _ => "product clicks and conversion",
    };
    format!(
        r#"Create exactly three distinctly different, ready-to-film TikTok product scripts in {lang}. The tone is {tone}; the goal is {goal_label}. Each script should run 25–35 seconds.

The three creative directions are:
1. relatable_problem — open on a recognizable frustration and resolve it with source-grounded features.
2. product_demo — open mid-demonstration and let observable product details carry the story.
3. quick_discovery — a concise creator-style discovery focused on one specific useful detail, without claiming personal use or endorsement.

Return ONLY this JSON object:
{{"scripts":[{{
  "title":"short human-readable concept title",
  "angle":"relatable_problem|product_demo|quick_discovery",
  "language":"en|es",
  "hook":"spoken opening, maximum 14 words",
  "voiceover":"complete natural spoken script",
  "scenes":[
    {{"seconds":"0–3s","visual":"simple phone-filmable action","narration":"spoken line","chinese_translation":"accurate Simplified Chinese translation of narration","on_screen_text":"short complementary overlay"}},
    {{"seconds":"3–10s","visual":"...","narration":"...","chinese_translation":"...","on_screen_text":"..."}},
    {{"seconds":"10–22s","visual":"...","narration":"...","chinese_translation":"...","on_screen_text":"..."}},
    {{"seconds":"22–30s","visual":"...","narration":"...","chinese_translation":"...","on_screen_text":"..."}}
  ],
  "caption":"one conversational caption",
  "hashtags":["3–5 specific relevant tags"],
  "disclosure":"short affiliate or sponsorship disclosure reminder"
}}]}}

Creative rules:
- The opening visual, spoken hook, and overlay must add three complementary pieces of information.
- Keep every shot achievable by one person with a phone and the product. Avoid production jargon.
- Write like a creator talking to one person: short lines, specific language, no corporate copy.
- Use ONLY the original product facts for factual claims. The analysis may guide framing but cannot add facts.
- Never claim personal ownership, testing, use, preference, endorsement, or results.
- Never invent use cases, price, discount, material, dimensions, ingredients, compatibility, certification, performance, popularity, comparison, safety, or results.
- Respect every claim boundary in the analysis. Do not say viral, best, perfect, guaranteed, life-changing, or use #fyp/#viral.
- For Spanish, write native neutral Latin-American creator copy rather than translating English literally.
- chinese_translation must faithfully translate each narration line into natural Simplified Chinese without adding claims.

Original product facts:
{}

Structured creative analysis:
{}"#,
        serde_json::to_string(product).unwrap(),
        serde_json::to_string(insights).unwrap()
    )
}

const SYSTEM_PROMPT: &str = r#"You are a direct-response TikTok UGC creative strategist for e-commerce products. Write product scripts that sound like a real creator, not a polished advertisement. Accuracy is mandatory: if a claim is absent from the supplied product facts, omit it. Never invent medical, financial, legal, safety, environmental, performance, or comparative claims. Return strict JSON only, without markdown or explanation."#;

const ANALYSIS_SYSTEM_PROMPT: &str = r#"You are a product positioning strategist. Your job is to transform sparse e-commerce listing facts into a structured creative brief while keeping facts and inferences visibly separate. Never fill missing product information with assumptions. Return strict JSON only, without markdown or explanation."#;

struct ApiError {
    status: StatusCode,
    message: String,
}
impl ApiError {
    fn bad_request(m: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: m.into(),
        }
    }
    fn bad_gateway(m: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: m.into(),
        }
    }
    fn service_unavailable(m: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: m.into(),
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

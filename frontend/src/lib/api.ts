export type Product = { asin: string; marketplace: string; source_url: string; title: string; main_image_url: string; bullet_points: string[] }
export type SellingPoint = { feature: string; customer_value: string; evidence: string; confidence: 'high' | 'medium' | 'low' }
export type TargetSegment = { segment: string; need: string; rationale: string; confidence: 'high' | 'medium' | 'low' }
export type ContentAngle = { name: string; insight: string; hook_direction: string; supporting_features: string[] }
export type ProductInsights = { product_summary: string; core_selling_points: SellingPoint[]; target_segments: TargetSegment[]; content_angles: ContentAngle[]; claim_boundaries: string[] }
export type Scene = { seconds: string; visual: string; narration: string; chinese_translation: string; on_screen_text?: string }
export type Script = { title: string; angle: string; language: 'en' | 'es'; hook: string; voiceover: string; scenes: Scene[]; caption: string; hashtags: string[]; disclosure: string }
export type CreativeResponse = { product: Product; insights: ProductInsights; scripts: Script[] }

async function request<T>(url: string, body: unknown): Promise<T> {
  let response: Response
  try {
    response = await fetch(url, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) })
  } catch {
    throw new Error('暂时无法连接创作服务，请确认后端正在运行。')
  }

  const raw = await response.text()
  if (!raw.trim()) {
    throw new Error(response.ok ? '服务没有返回结果，请重新生成。' : `创作服务暂时无响应（${response.status}），请稍后重试。`)
  }

  let data: unknown
  try {
    data = JSON.parse(raw)
  } catch {
    throw new Error(`创作服务返回了无法识别的内容（${response.status}），请重新生成。`)
  }

  if (!response.ok) {
    const message = typeof data === 'object' && data !== null && 'error' in data ? String(data.error) : '请求失败，请重试。'
    throw new Error(message)
  }
  return data as T
}
export const generateCreative = (input: string, marketplace: string, language: 'en' | 'es', goal: string, tone: string) => request<CreativeResponse>('/api/creative/generate', { input, marketplace, language, goal, tone })

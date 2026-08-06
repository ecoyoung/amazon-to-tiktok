<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { generateCreative, type CreativeResponse, type Script } from './lib/api'

const productInput = ref('')
const marketplace = ref('com')
const language = ref<'en' | 'es'>('en')
const goal = ref('conversion')
const tone = ref('ugc')
const result = ref<CreativeResponse | null>(null)
const selectedScript = ref(0)
const loading = ref(false)
const stage = ref(0)
const error = ref('')
let progressTimer: ReturnType<typeof setInterval> | undefined

const stages = ['读取商品信息', '提炼购买理由', '构思三个方向', '完成拍摄脚本']
const activeScript = computed(() => result.value?.scripts[selectedScript.value] ?? null)
const angleNames: Record<string, string> = {
  relatable_problem: '痛点切入',
  product_demo: '产品演示',
  quick_discovery: '快速种草',
}

async function createScripts() {
  if (!productInput.value.trim() || loading.value) return
  loading.value = true; error.value = ''; result.value = null; selectedScript.value = 0; stage.value = 0
  progressTimer = setInterval(() => { if (stage.value < stages.length - 1) stage.value += 1 }, 2600)
  try { result.value = await generateCreative(productInput.value, marketplace.value, language.value, goal.value, tone.value); stage.value = stages.length - 1 }
  catch (e) { error.value = e instanceof Error ? e.message : '暂时没有生成成功，请稍后再试。' }
  finally { loading.value = false; if (progressTimer) clearInterval(progressTimer) }
}

async function copyVoiceover(script: Script | null) {
  if (!script) return
  const rows = script.scenes.map(scene => `| ${markdownCell(scene.seconds)} | ${markdownCell(scene.narration)} | ${markdownCell(scene.chinese_translation)} |`).join('\n')
  await navigator.clipboard.writeText(`# ${script.title} · 口播稿\n\n> **Hook：** ${script.hook}\n\n| **时间段** | **口播** | **中文翻译** |\n|---|---|---|\n${rows}`)
}

async function copyFullScript(script: Script | null) {
  if (!script) return
  const rows = script.scenes.map(scene => `| ${markdownCell(scene.seconds)} | ${markdownCell(scene.visual)} | ${markdownCell(scene.narration)} | ${markdownCell(scene.chinese_translation)} | ${markdownCell(scene.on_screen_text || '—')} |`).join('\n')
  const markdown = `# ${script.title}\n\n> **Hook：** ${script.hook}\n\n| **时间段** | **画面** | **口播** | **中文翻译** | **屏幕字幕** |\n|---|---|---|---|---|\n${rows}\n\n## 发布文案\n\n${script.caption}\n\n${script.hashtags.join(' ')}\n\n> ${script.disclosure}`
  await navigator.clipboard.writeText(markdown)
}

function markdownCell(value: string) {
  return value.replace(/\|/g, '\\|').replace(/\r?\n/g, '<br>').trim()
}

onBeforeUnmount(() => { if (progressTimer) clearInterval(progressTimer) })
</script>

<template>
  <div class="app-shell">
    <nav class="topbar"><a class="brand" href="#"><img src="/hookpop-icon.png" alt="" />HookPop</a><span>TikTok</span></nav>

    <main>
      <section class="hero" :class="{ compact: result || loading }">
        <p class="kicker">AMAZON · TikTok</p>
        <h1>一个ASIN，从卖点到创意<br><span>直接开拍</span></h1>

        <form class="composer" @submit.prevent="createScripts">
          <div class="input-row">
            <input v-model="productInput" aria-label="Amazon 商品链接或 ASIN" placeholder="粘贴 Amazon 商品链接或输入 ASIN" autocomplete="off" />
            <button type="submit" :disabled="loading || !productInput.trim()">{{ loading ? '正在创作' : '生成 3 个创意' }}</button>
          </div>
          <div class="options-row">
            <label class="country-control"><span>站点</span><select v-model="marketplace" aria-label="Amazon 站点"><option value="com">🇺🇸 US</option><option value="co.uk">🇬🇧 UK</option><option value="de">🇩🇪 DE</option><option value="fr">🇫🇷 FR</option><option value="co.jp">🇯🇵 JP</option><option value="ca">🇨🇦 CA</option><option value="it">🇮🇹 IT</option><option value="es">🇪🇸 ES</option><option value="com.mx">🇲🇽 MX</option></select></label>
            <label><span>脚本语言</span><select v-model="language"><option value="en">English</option><option value="es">Español</option></select></label>
            <label><span>内容目标</span><select v-model="goal"><option value="conversion">带货转化</option><option value="organic">自然互动</option><option value="awareness">产品认知</option></select></label>
            <label><span>表达风格</span><select v-model="tone"><option value="ugc">自然真实</option><option value="energetic">轻快有力</option><option value="premium">克制高级</option></select></label>
          </div>
        </form>
      </section>

      <p v-if="error" class="notice-error">{{ error }}</p>

      <section v-if="loading" class="progress-card">
        <div class="progress-head"><div class="spinner"></div><div><strong>{{ stages[stage] }}</strong><p>正在把商品信息变成可以直接执行的创意。</p></div></div>
        <div class="progress-track"><span :style="{ width: `${((stage + 1) / stages.length) * 100}%` }"></span></div>
        <div class="progress-labels"><span v-for="(item, index) in stages" :key="item" :class="{ done: index <= stage }">{{ item }}</span></div>
      </section>

      <template v-if="result && activeScript">
        <section class="product-bar">
          <img v-if="result.product.main_image_url" :src="result.product.main_image_url" :alt="result.product.title" />
          <div><span>{{ result.product.asin }}</span><h2>{{ result.product.title }}</h2></div>
          <button class="quiet-button" @click="createScripts">换一组创意</button>
        </section>

        <section class="results-heading"><div><p class="kicker">3 个创意方向</p><h2>选择一个开始拍</h2></div><p>每个方向都来自同一组商品事实，但采用不同的开场和叙事方式。</p></section>

        <section class="concept-grid">
          <button v-for="(script, index) in result.scripts" :key="script.angle" class="concept-card" :class="{ active: selectedScript === index }" @click="selectedScript = index">
            <span class="concept-number">0{{ index + 1 }}</span><span class="concept-angle">{{ angleNames[script.angle] || script.angle }}</span>
            <strong>{{ script.title }}</strong><p>“{{ script.hook }}”</p><span class="choose-label">{{ selectedScript === index ? '正在查看' : '查看脚本' }}</span>
          </button>
        </section>

        <section class="script-workspace">
          <div class="script-main">
            <div class="script-header"><div><span class="angle-pill">{{ angleNames[activeScript.angle] || activeScript.angle }}</span><h2>{{ activeScript.title }}</h2></div><div class="script-actions"><button class="quiet-button" @click="copyVoiceover(activeScript)">复制口播</button><button class="primary-small" @click="copyFullScript(activeScript)">复制完整脚本</button></div></div>
            <div class="hook-block"><span>开场 Hook</span><p>{{ activeScript.hook }}</p></div>
            <div class="scene-list">
              <article v-for="scene in activeScript.scenes" :key="scene.seconds" class="scene-row"><time>{{ scene.seconds }}</time><div class="scene-visual"><span>画面</span><p>{{ scene.visual }}</p></div><div class="scene-copy"><span>口播</span><p>{{ scene.narration }}</p><small class="translation">中文 · {{ scene.chinese_translation }}</small><small v-if="scene.on_screen_text">屏幕字幕 · {{ scene.on_screen_text }}</small></div></article>
            </div>
            <div class="publishing-copy"><div><span>发布文案</span><p>{{ activeScript.caption }}</p></div><p class="hashtags">{{ activeScript.hashtags.join(' ') }}</p></div>
          </div>

          <aside class="brief-panel">
            <span class="aside-label">创作依据</span><h3>{{ result.insights.product_summary }}</h3>
            <ul><li v-for="point in result.insights.core_selling_points.slice(0, 4)" :key="point.feature"><strong>{{ point.feature }}</strong><span>{{ point.customer_value }}</span></li></ul>
            <details><summary>查看内容边界</summary><p v-for="item in result.insights.claim_boundaries" :key="item">{{ item }}</p></details>
          </aside>
        </section>
      </template>
    </main>

    <footer>HookPop · Product facts in, film-ready ideas out.</footer>
  </div>
</template>

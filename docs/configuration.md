# 环境配置

后端从 `backend/.env` 读取配置。不要把真实密钥提交到版本控制。

## 本地开发（推荐）

复制示例文件：

```bash
cd /Users/anker/Documents/curl/backend
cp .env.example .env
```

填写 `backend/.env`：

```env
DEEPSEEK_API_KEY=sk-你的-deepseek-key
KEEPA_API_KEY=你的-keepa-key

# 以下两项通常保持默认
LLM_MODEL=deepseek-v4-flash
LLM_BASE_URL=https://api.deepseek.com
```

`backend/.env` 已被 `.gitignore` 忽略，不要提交真实密钥。

## 部署环境

部署时可以不使用 JSON 文件，直接注入环境变量：

```env
DEEPSEEK_API_KEY=sk-你的-deepseek-key
KEEPA_API_KEY=你的-keepa-key
LLM_MODEL=deepseek-v4-flash
LLM_BASE_URL=https://api.deepseek.com
```

后端仍兼容 `KEEPA_KEY_FILE` 文件配置，但仅作为旧环境的备用方式；如果两者同时存在，优先使用 `KEEPA_API_KEY`。

## 验证配置

启动后端后执行：

```bash
curl -s http://127.0.0.1:8787/api/health | jq
```

正确结果应包含：

```json
{
  "status": "ok",
  "keepa_configured": true,
  "deepseek_configured": true,
  "llm_model": "deepseek-v4-flash"
}
```

健康检查只返回配置状态，不会返回密钥内容。

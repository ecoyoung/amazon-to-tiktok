# 环境配置

后端从 `backend/.env` 读取配置。不要把真实密钥提交到版本控制。

使用 Docker Compose 时，同一个文件会通过 `env_file` 注入后端容器，无需维护第二份配置。

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
TUNNEL_TOKEN=你的-Cloudflare-Tunnel-token

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

`TUNNEL_TOKEN` 由 Compose 中的 `cloudflared` 容器读取。换服务器时复制 `backend/.env`，然后运行 `docker compose up -d --build` 即可启动同一个 Tunnel。请始终把它当作密码保存，不要提交到 Git。

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

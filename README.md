# HookPop

一个本地工具：输入 Amazon ASIN 或商品链接，通过 Keepa Product API 获取主图、标题和五点描述并输出规范化商品资料。随后通过两次独立 LLM 调用，先形成结构化卖点与内容切入点，再生成英文或西语 TikTok 视频脚本。

## 生成流程

用户只需调用一次 `POST /api/creative/generate` 或在页面提交一次：

1. 后台抓取并标准化 Amazon 商品事实。
2. 第一次 LLM 调用分析核心卖点、消费价值、潜在人群、内容角度和禁止宣称边界。
3. 第二次 LLM 调用基于事实与分析结果，一次生成痛点切入、产品演示和快速种草三个方向。

卖点分析默认收进结果页的“创作依据”，用户直接选择脚本，无需操作中间流程。

页面支持以国旗和站点代码选择 US、UK、DE、FR、JP、CA、IT、ES 和 MX；粘贴完整 Amazon 链接时会优先采用链接中的站点。

生成结果包含每个镜头的中文翻译。“复制口播”输出 Markdown 口播表，“复制完整脚本”输出以下 Markdown 列：时间段、画面、口播、中文翻译、屏幕字幕。

## 架构

- `frontend/`：Vue 3 + TypeScript + Vite
- `backend/`：Rust + Axum API，直接调用 Keepa 和 DeepSeek

## 快速开始

1. 配置后端：

   ```bash
   cd backend
   cp .env.example .env
   # 按 docs/configuration.md 填写 DeepSeek 和 Keepa 配置。
   cargo run
   ```

2. 启动前端：

   ```bash
   cd frontend
   npm install
   npm run dev
   ```

访问 http://127.0.0.1:5173。详细配置见 [docs/configuration.md](docs/configuration.md)，代码结构见 [docs/architecture.md](docs/architecture.md)。

## Docker 启动

确认 `backend/.env` 已配置 DeepSeek、Keepa 和 `TUNNEL_TOKEN` 后，在项目根目录执行：

```bash
docker compose up -d --build
```

访问 http://127.0.0.1:5173；配置好 Cloudflare Public Hostname 后也可通过对应域名访问。查看状态和日志：

```bash
docker compose ps
docker compose logs -f
```

停止服务：

```bash
docker compose down
```

## 注意

- 商品数据来自 Keepa Product API。每次查询会消耗 Keepa token，应在生产环境加入按 ASIN/站点缓存。
- `keepa_key.json` 已加入 `.gitignore`，不要提交或发送其中的密钥。
- `DEEPSEEK_API_KEY` 只保留在 Rust 后端，绝不发送到浏览器。默认使用 `deepseek-v4-flash`，也可通过 `LLM_MODEL` 切换模型。
- 缺少任一必需密钥时，接口会返回明确配置错误，不会生成伪造的本地结果。

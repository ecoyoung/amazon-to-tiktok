# 项目架构

```text
curl/
├── backend/                 Rust + Axum API
│   ├── src/main.rs          配置、Keepa、DeepSeek 与创意编排
│   ├── .env.example         环境变量模板
│   └── Cargo.toml
├── frontend/                Vue 3 + TypeScript + Vite
│   ├── public/              HookPop 图标等静态资源
│   └── src/
│       ├── App.vue          产品界面与交互
│       ├── lib/api.ts       唯一前端 API 客户端
│       └── style.css        页面样式
├── docs/                    配置与架构文档
├── compose.yaml             Docker Compose 生产式本地运行入口
├── keepa_key.json           旧版 Keepa 密钥备份（已忽略，可移除）
├── PROMPT_RESEARCH.md       提示词研究记录
├── lessons.md               已验证问题与修复
└── lessons-summary.md       问题索引
```

## 请求流程

1. Vue 只调用 `POST /api/creative/generate`。
2. Rust 规范化 ASIN 与站点，并读取六小时商品缓存。
3. 缓存未命中时调用 Keepa Product API，映射标题、主图和五点。
4. 第一次 DeepSeek 请求生成结构化卖点与内容角度。
5. 第二次 DeepSeek 请求生成三条脚本；事实校验失败时在后端自动修复。
6. 前端展示镜头、口播、中文翻译和 Markdown 复制结果。

产品 API 保持单入口，避免前端依赖内部 LLM 编排步骤。

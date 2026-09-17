**English** | [简体中文](README.md)

# HookPop

A local tool: enter an Amazon ASIN or product link, fetch the main image, title, and bullet points via the Keepa Product API, and output a normalized product profile. Then, through two independent LLM calls, it first produces structured selling points and content angles, and then generates a TikTok video script in English or Spanish.

## Generation Flow

Users only need to call `POST /api/creative/generate` once or submit once on the page:

1. The backend fetches and standardizes Amazon product facts.
2. The first LLM call analyzes core selling points, consumer value, potential audiences, content angles, and prohibited-claim boundaries.
3. The second LLM call builds on the facts and analysis to generate three directions in one pass: pain-point hook, product demonstration, and quick product recommendation.

The selling-point analysis is tucked into the "Creative Rationale" section of the results page by default; users simply pick a script without touching any intermediate steps.

The page supports selecting US, UK, DE, FR, JP, CA, IT, ES, and MX via flags and site codes; when a full Amazon link is pasted, the marketplace in the link takes priority.

The generated results include a Chinese translation for each shot. "Copy Voiceover" outputs a Markdown voiceover table, while "Copy Full Script" outputs a Markdown table with the following columns: time range, visuals, voiceover, Chinese translation, and on-screen captions.

## Architecture

- `frontend/`: Vue 3 + TypeScript + Vite
- `backend/`: Rust + Axum API that calls Keepa and DeepSeek directly

## Quick Start

1. Configure the backend:

   ```bash
   cd backend
   cp .env.example .env
   # 按 docs/configuration.md 填写 DeepSeek 和 Keepa 配置。
   cargo run
   ```

2. Start the frontend:

   ```bash
   cd frontend
   npm install
   npm run dev
   ```

Visit http://127.0.0.1:5173. See [docs/configuration.md](docs/configuration.md) for detailed configuration and [docs/architecture.md](docs/architecture.md) for the code structure.

## Starting with Docker

After confirming that `backend/.env` has DeepSeek, Keepa, and `TUNNEL_TOKEN` configured, run the following from the project root:

```bash
docker compose up -d --build
```

Visit http://127.0.0.1:5173; once a Cloudflare Public Hostname is configured, it is also accessible via the corresponding domain. Check status and logs:

```bash
docker compose ps
docker compose logs -f
```

Stop the services:

```bash
docker compose down
```

## Notes

- Product data comes from the Keepa Product API. Every query consumes Keepa tokens, so a per-ASIN/marketplace cache should be added in production.
- `keepa_key.json` is already in `.gitignore`; do not commit or share the keys inside it.
- `DEEPSEEK_API_KEY` stays in the Rust backend only and is never sent to the browser. It defaults to `deepseek-v4-flash`, and the model can be switched via `LLM_MODEL`.
- If any required key is missing, the API returns an explicit configuration error instead of generating fake local results.

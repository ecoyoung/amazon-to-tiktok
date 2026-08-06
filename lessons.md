# Engineering lessons

## Python scraper started outside the project virtual environment

- Symptom: ASIN scraping returned `ModuleNotFoundError: No module named 'bs4'` from `amazon_scraper.py`.
- Root cause: an older Rust backend process was still running with the default system `python3`; it had started before `SCRAPER_COMMAND` was configured to use the project virtual environment.
- Fix: install scraper dependencies in `.venv`, set `SCRAPER_COMMAND=../.venv/bin/python3`, and restart the stale backend process.
- Validation: importing `bs4` and `curl_cffi` with `.venv/bin/python3` succeeded; a valid-format ASIN reached the remote HTTP request instead of failing during Python imports.
- Prevention: keep the interpreter path explicit in backend configuration and restart services after environment changes. Future health checks should report the active scraper interpreter/dependency status.

## DeepSeek JSON mode returned an empty message body

- Symptom: script generation returned `模型返回的文案格式不正确`, while the DeepSeek HTTP request itself succeeded.
- Root cause: `deepseek-v4-flash` enables thinking mode by default, and DeepSeek JSON Output can occasionally return an empty `message.content`. The backend treated the empty string as malformed JSON and discarded the specific parse reason.
- Fix: explicitly disable thinking mode for structured copy generation, retry one time when content is empty or invalid, accept fenced JSON defensively, and return a precise non-sensitive validation error after retries.
- Validation: the same English and Spanish test-product requests returned HTTP 200 with a parsed four-scene script after the backend restart.
- Prevention: treat model HTTP success, non-empty content, JSON syntax, schema validation, and claim validation as separate quality gates. Preserve non-sensitive failure reasons for diagnosis.

## Frontend JSON parsing masked an empty upstream response

- Symptom: the page displayed `Failed to execute 'json' on 'Response': Unexpected end of JSON input` instead of a useful product error.
- Root cause: the frontend called `response.json()` unconditionally. An empty response body therefore raised a browser parsing exception before HTTP status or backend context could be shown. The source of the one observed empty upstream response is pending validation; it may have occurred during a backend restart or proxy disconnect.
- Fix: read the response as text first, handle network, empty-body, non-JSON, HTTP-error, and success cases separately, and return concise user-facing messages.
- Validation: the frontend production build passed, backend health returned 200, and a request through the Vite proxy preserved the backend's JSON 400 error correctly.
- Prevention: API clients must not assume every HTTP response contains JSON, including successful and proxy-generated responses.

## Internal script quality gate leaked into the user experience

- Symptom: generation stopped with `包含无法由产品资料证明的表述：i use`.
- Root cause: the three-script orchestration validated unsupported first-person experience claims only after generation and returned the validator's internal diagnostic directly to the user.
- Fix: keep the factual quality gate, but automatically regenerate the complete script bundle up to three times with the exact failed constraint and neutral-demonstration rewrite instructions. Only a concise product-level error is returned if every repair attempt fails.
- Validation: backend compilation and health checks passed after restart. End-to-end validation against the user's original product is pending the next generation attempt.
- Prevention: recoverable model-quality failures should trigger bounded repair loops; internal validator strings should not be exposed as normal product errors.

## Live Amazon HTML scraping was too unstable for the product path

- Symptom: ASIN generation was repeatedly interrupted by Python environment mismatches, Amazon HTTP failures, verification pages, and variable page markup.
- Root cause: the core workflow depended on spawning a Python `curl_cffi` process and parsing a live retail page that is not a stable data contract.
- Fix: replace the scraper subprocess with a direct Rust integration to Keepa Product API. Map locale domains, normalize `title`, `features`, and `images`, strip occasional HTML, protect the key outside version control, and cache products by locale plus ASIN for six hours.
- Validation: the official example ASIN returned HTTP 200 with a title, Amazon image URL, and four feature bullets; the full product-to-analysis-to-three-scripts workflow returned HTTP 200 with four scenes per script. A repeated product request returned identical data from cache in about 1 ms.
- Prevention: use a documented catalog API for product facts; keep page scraping out of the critical user path. Enable response decompression because Keepa returns gzip-encoded JSON.

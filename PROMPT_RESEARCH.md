# TikTok UGC script prompt research

The production prompt in `backend/src/main.rs` combines the most reusable patterns found across public TikTok/short-form script resources:

1. **Hook → body/demonstration → payoff/CTA.** The first 1–3 seconds must create a concrete reason to keep watching.
2. **Three-layer hooks.** Treat the opening visual, spoken line, and on-screen text as separate information channels instead of repeating the same sentence.
3. **Hook competition before drafting.** Generate several distinct angles internally, choose the strongest fact-grounded one, and only then draft the final script.
4. **Scene-level production detail.** Return timing, a phone-filmable action, narration, and a complementary text overlay.
5. **Conversational delivery.** Short spoken sentences and natural language outperform generic ad copy as a filming brief.
6. **Grounding and quality gates.** Product-page facts are the sole source of truth; reject fabricated prices, results, certifications, comparisons, urgency, reviews, or trend claims.
7. **Language localization.** Spanish output is written as neutral Latin-American creator copy, not translated literally from English.

Public references:

- https://byword.ai/templates/social-media/tiktok-script
- https://scrollscript.ai/blog/short-form-video-script-template
- https://www.hookmafia.io/ugc-script-generator
- https://ugccreatorcompanion.app/
- https://github.com/bytesagain/ai-skills
- https://api-docs.deepseek.com/zh-cn/
- https://api-docs.deepseek.com/zh-cn/guides/json_mode/

These sources informed the structure and constraints; their wording was not copied into the production prompt.

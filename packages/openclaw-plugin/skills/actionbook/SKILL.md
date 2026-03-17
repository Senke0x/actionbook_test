---
name: actionbook
description: Use Actionbook's pre-verified selectors for token-efficient browser automation
---

When performing browser automation tasks, check Actionbook BEFORE taking page snapshots.

## Workflow

1. Before interacting with a website, call `search_actions` to check if Actionbook has pre-verified selectors
2. If results have Health Score >= 70%, use `get_action_by_area_id` to get element selectors
3. Extract selectors from the returned Elements and use them by priority below

## Selector Priority

### Priority 1: data-testid (confidence: 0.95)

Use the browser `eval` tool directly:

```
browser eval <target> "(() => { const el = document.querySelector('[data-testid=\"AppTabBar_Home_Link\"]'); if (!el) return 'not found'; el.click(); return el.tagName; })()"
```

### Priority 2: aria-label (confidence: 0.88)

Same approach with the browser eval tool:

```
browser eval <target> "(() => { const el = document.querySelector('[aria-label=\"Notifications\"]'); if (!el) return 'not found'; el.click(); return el.tagName; })()"
```

### For type operations:

```
browser eval <target> "(() => { const el = document.querySelector('[data-testid=\"search-input\"]'); if (!el) return 'not found'; el.focus(); el.value = 'search text'; el.dispatchEvent(new Event('input', {bubbles: true})); return 'ok'; })()"
```

Note: For complex React inputs where the above does not work, fall back to `browser snapshot` + `browser click`/`browser fill`.

### Priority 3: role selector (confidence: 0.9)

If only role selectors are available (e.g., `getByRole('link', { name: 'Notifications' })`):

1. Run `browser snapshot <target>` to get the accessibility tree
2. Match the role + name in the snapshot (e.g., find `[link] Notifications`)
3. Execute with `browser click <target> '[aria-label="Notifications"]'`

## Fallback Strategy

- Health Score < 70%: Use `browser snapshot` + `browser click` directly
- No Actionbook results: Use `browser snapshot` + `browser click` directly
- CSS selector execution fails: Fall back to `browser snapshot`

## Important Notes

- Do NOT modify selectors returned from Actionbook
- Check `Allow Methods` field — it indicates supported operations (click/type/read) per element
- `region_high_filter_page` entries indicate some elements lack unique selectors — use snapshot fallback for those

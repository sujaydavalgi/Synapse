# Davalgi / Spanda landing page

Static single-page site for **davalgi.com** (or **spanda.davalgi.com**) until a full marketing site exists.

Source copy lives in [docs/website-content.md](../docs/website-content.md).

## Deploy

### Cloudflare Pages (recommended)

1. Cloudflare dashboard → **Workers & Pages** → **Create** → **Pages** → **Connect to Git**
2. Select **Davalgi/Spanda**, branch `main`, build output directory: **`website`**
3. Custom domain: `davalgi.com` (and optionally `www.davalgi.com`)
4. No build command needed — static HTML only

### Netlify

- Publish directory: `website`
- Or drag-and-drop the `website/` folder at [app.netlify.com/drop](https://app.netlify.com/drop)

### GitHub Pages (org site)

If you prefer a separate **Davalgi/davalgi.com** repo, copy `index.html` there and enable Pages from `main`.

## After deploy

1. Set GitHub repo **Website** to `https://davalgi.com`
2. Update `Cargo.toml` / `package.json` `homepage` if you want crates/npm to point at the domain (optional; GitHub remains fine for now)

## Local preview

```bash
python3 -m http.server 8080 --directory website
```

Open [http://localhost:8080](http://localhost:8080).

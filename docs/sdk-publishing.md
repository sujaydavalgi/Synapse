# Publishing the official SDKs

How to publish **Spanda Rust, Python, and TypeScript** SDKs from this repository using GitHub Actions.

## Packages and registries

| SDK | Source path | Registry package | Release tag pattern |
|-----|-------------|------------------|---------------------|
| Rust | `crates/spanda-sdk/` | [`spanda-sdk`](https://crates.io/crates/spanda-sdk) | `crates-sdk-vX.Y.Z` |
| Python | `sdk/python/` | [`spanda-sdk`](https://pypi.org/project/spanda-sdk/) | `sdk-python-vX.Y.Z` |
| TypeScript | `sdk/typescript/` | [`@davalgi-spanda/sdk`](https://www.npmjs.com/package/@davalgi-spanda/sdk) | `npm-sdk-vX.Y.Z` |

Version numbers live in:

- `Cargo.toml` workspace version → `crates/spanda-sdk`
- `sdk/python/pyproject.toml` → `[project].version`
- `sdk/typescript/package.json` → `"version"`

Tag suffix must match the version you intend to ship (for example `sdk-python-v0.4.0` when `version = "0.4.0"`).

## One-time setup: GitHub secrets

Both publish workflows read **repository secrets** (not committed files).

1. Open the Spanda repo on GitHub.
2. Go to **Settings → Secrets and variables → Actions**.
3. Add:

| Secret name | Used by | Purpose |
|-------------|---------|---------|
| `CRATES_IO_TOKEN` | [publish-sdk-rust.yml](../.github/workflows/publish-sdk-rust.yml) | Upload `spanda-sdk` to crates.io |
| `PYPI_API_TOKEN` | [publish-sdk-python.yml](../.github/workflows/publish-sdk-python.yml) | Upload wheels to PyPI |
| `NPM_TOKEN` | [publish-sdk-typescript.yml](../.github/workflows/publish-sdk-typescript.yml) | Upload `@davalgi-spanda/sdk` to npm |

You need **admin** (or equivalent) access on the repository to create secrets.

**Never** commit tokens to the repo, paste them in issues/PRs, or store them in `.pypirc` / `.npmrc` under version control.

---

## crates.io (`CRATES_IO_TOKEN`)

### Create a crates.io account and token

1. Register at [crates.io](https://crates.io/) (GitHub login works).
2. **Account settings → API Tokens** → **New Token**.
3. Name it (for example `spanda-github-actions`) and copy the token.

Tokens expire (for example **365 days**). Set a calendar reminder to create a new token and update `CRATES_IO_TOKEN` in GitHub before expiry.

### Add to GitHub

- **Name:** `CRATES_IO_TOKEN`
- **Value:** the crates.io API token

### Workflow behavior

On tag push `crates-sdk-v*`:

1. `cargo test -p spanda-sdk` (with and without `grpc` feature)
2. `cargo package -p spanda-sdk`
3. `cargo publish -p spanda-sdk`

Manual trigger: **Actions → Publish Rust SDK → Run workflow** (tests/package only; upload on matching tags).

Install after publish:

```bash
cargo add spanda-sdk
# optional gRPC client
cargo add spanda-sdk --features grpc
```

---

## PyPI (`PYPI_API_TOKEN`)

### Create a PyPI account and token

1. Register at [pypi.org](https://pypi.org/account/register/) if needed.
2. Log in → **Account settings → API tokens**.
3. **Add API token**:
   - **Scope:** entire account (first publish) or project `spanda-sdk` after it exists.
   - Copy the token immediately — PyPI shows it once. It starts with `pypi-AgEI...`.

For PyPI uploads, the username is always `__token__`; GitHub Actions only needs the **token string** as the secret value.

### Add to GitHub

- **Name:** `PYPI_API_TOKEN`
- **Value:** the full `pypi-...` token (not `__token__`)

### Workflow behavior

On tag push `sdk-python-v*`:

1. Runs `pytest sdk/python/tests`
2. Builds a wheel/sdist in `sdk/python/dist`
3. Publishes via [pypa/gh-action-pypi-publish](https://github.com/pypa/gh-action-pypi-publish)

Manual trigger: **Actions → Publish Python SDK → Run workflow** (tests and build always run; upload only on matching tags).

---

## npm (`NPM_TOKEN`)

### Why `@davalgi-spanda` and not `@spanda`

The npm organization name **`spanda`** is already taken by another account. Scoped packages must live under an org or user you control. This repo publishes TypeScript as:

```text
@davalgi-spanda/sdk
```

Create the org at [npmjs.com/org/create](https://www.npmjs.com/org/create) if needed. **Public packages under an org are free** — the org itself does not expire.

### Create an npm access token

1. Log in at [npmjs.com](https://www.npmjs.com/).
2. **Access Tokens** → [npmjs.com/settings/~/tokens](https://www.npmjs.com/settings/~/tokens).
3. **Generate New Token → Granular Access Token**:
   - **Permissions:** Read and write
   - **Packages/scopes:** `@davalgi-spanda/*` (or the specific package)
   - **Expiration:** up to **90 days** (npm maximum for write tokens)
   - **Bypass 2FA:** **enable** — required for non-interactive publish from GitHub Actions

Copy the token (`npm_...`).

### 90-day token rotation

The **organization does not expire** — only the **access token** does. npm caps write-enabled granular tokens at 90 days.

Before expiry:

1. Create a new granular token with the same permissions.
2. Update **`NPM_TOKEN`** in GitHub **Settings → Secrets and variables → Actions**.
3. Revoke the old token on npm.

Set a calendar reminder (~80 days) so CI publish does not fail mid-release.

### Add to GitHub

- **Name:** `NPM_TOKEN`
- **Value:** the full `npm_...` token

### Package manifest

`sdk/typescript/package.json` must include:

```json
{
  "name": "@davalgi-spanda/sdk",
  "publishConfig": { "access": "public" }
}
```

### Workflow behavior

On tag push `npm-sdk-v*`:

1. `npm ci`, `npm test`, `npm run build` under `sdk/typescript`
2. `npm publish --access public`

Manual trigger: **Actions → Publish TypeScript SDK → Run workflow**.

---

## Local dry-run (no publish)

From repo root:

```bash
./scripts/verify_sdk_publish_ready.sh
```

This runs Python tests, builds a PyPI wheel, runs TypeScript tests/build, and checks related packages — without uploading.

Individual checks:

```bash
# Python
pip install -e "sdk/python[dev]"
pytest sdk/python/tests -q
(cd sdk/python && python -m build)

# TypeScript
npm ci --prefix sdk/typescript
npm test --prefix sdk/typescript
npm run build --prefix sdk/typescript
npm pack --prefix sdk/typescript
```

---

## Release a new version

### 1. Bump versions

Update version in **both** manifest files if releasing both SDKs together:

- `sdk/python/pyproject.toml`
- `sdk/typescript/package.json` (and run `npm install --prefix sdk/typescript` if lockfile needs refresh)

Commit and push to `main`.

### 2. Tag and push

From repo root:

```bash
# Python → PyPI
git tag sdk-python-v0.4.0
git push origin sdk-python-v0.4.0

# Rust → crates.io
git tag crates-sdk-v0.4.0
git push origin crates-sdk-v0.4.0

# TypeScript → npm
git tag npm-sdk-v0.4.1
git push origin npm-sdk-v0.4.1
```

Tags can be pushed in one command:

```bash
git push origin sdk-python-v0.4.0 npm-sdk-v0.4.0
```

### 3. Watch CI

**GitHub → Actions** — confirm **Publish Rust SDK**, **Publish Python SDK**, and **Publish TypeScript SDK** succeed.

### 4. Verify install

```bash
pip install spanda-sdk
python -c "from spanda_sdk import SpandaClient; print(SpandaClient.local())"

cargo add spanda-sdk

npm install @davalgi-spanda/sdk
node -e "import('@davalgi-spanda/sdk').then(m => console.log(m.SpandaClient.local()))"
```

Check registry pages:

- [crates.io/crates/spanda-sdk](https://crates.io/crates/spanda-sdk)
- [pypi.org/project/spanda-sdk/](https://pypi.org/project/spanda-sdk/)
- [npmjs.com/package/@davalgi-spanda/sdk](https://www.npmjs.com/package/@davalgi-spanda/sdk)

Set a calendar reminder before each token expires and update the GitHub secret.

| Registry | Secret | Typical expiry |
|----------|--------|----------------|
| crates.io | `CRATES_IO_TOKEN` | Up to 365 days (set at token creation) |
| PyPI | `PYPI_API_TOKEN` | Per token policy on pypi.org |
| npm | `NPM_TOKEN` | Up to 90 days (write granular tokens) |

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| crates.io 400 verified email | Email not verified on crates.io account | [crates.io/settings/profile](https://crates.io/settings/profile) — verify inbox link; re-tag after confirmation |
| crates.io empty token | `CRATES_IO_TOKEN` missing in GitHub | Add secret; re-tag `crates-sdk-v*` |
| PyPI publish 403 / invalid credentials | Wrong or revoked `PYPI_API_TOKEN` | Regenerate token; update GitHub secret |
| npm publish 401 / E401 | Expired token or missing Bypass 2FA | New granular token with Bypass 2FA; update `NPM_TOKEN` |
| npm scope / org error | Package name not under your org | Use `@davalgi-spanda/sdk`, not `@spanda/sdk` |
| npm “organization name not available” | Name taken globally | Pick another org (for example `@davalgi-spanda`) |
| Tag pushed but no upload | Tag pattern mismatch | Use `sdk-python-v*` or `npm-sdk-v*` exactly |
| Version already exists on registry | Re-used tag/version | Bump `version` in manifest; new tag |
| Workflow green but package missing | Publish step skipped | Publish runs only on tag push, not `workflow_dispatch` alone |

If a token was exposed (chat, log, commit), **revoke it immediately** on PyPI/npm and create a new one.

---

## Related workflows

| Workflow | Tag | Package |
|----------|-----|---------|
| [publish-sdk-rust.yml](../.github/workflows/publish-sdk-rust.yml) | `crates-sdk-v*` | `spanda-sdk` |
| [publish-sdk-python.yml](../.github/workflows/publish-sdk-python.yml) | `sdk-python-v*` | `spanda-sdk` (PyPI) |
| [publish-sdk-typescript.yml](../.github/workflows/publish-sdk-typescript.yml) | `npm-sdk-v*` | `@davalgi-spanda/sdk` |
| [publish-npm-web.yml](../.github/workflows/publish-npm-web.yml) | `npm-web-v*` | `@davalgi-spanda/web` |

Legacy Python enterprise client: `packages/sdk-python` — not published by the canonical SDK workflow.

### Web panel (`@davalgi-spanda/web`)

Same `NPM_TOKEN` and org as the TypeScript SDK. Package lives in `packages/web/` with exports for `ControlCenterPanel` and `index.css`.

```bash
git tag npm-web-v0.4.0
git push origin npm-web-v0.4.0
```

See [packages/web/PUBLISHING.md](../packages/web/PUBLISHING.md).

## See also

- [sdk.md](./sdk.md) — SDK overview
- [sdk-python.md](./sdk-python.md) · [sdk-typescript.md](./sdk-typescript.md) — language guides
- [control-center-api.md](./control-center-api.md) — API surface the SDKs call

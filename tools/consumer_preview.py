"""Make a local review page from standalone consumer renderer captures."""
from pathlib import Path

root = Path(__file__).resolve().parents[1] / "target/visual-report/desktop-beta"
root.mkdir(parents=True, exist_ok=True)
expected = [f"{theme}-{size}-{scene}.png" for theme in ("light", "dark")
            for size in ("420-1", "1000-1.25", "1000-2")
            for scene in ("overview", "preferences", "editor", "discard")]
missing = [name for name in expected if not (root / name).is_file()]
if missing:
    raise SystemExit(f"Capture the consumer review first; missing {len(missing)} frames")
(root / "index.html").write_text('''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Desktop beta review · Northstar Studio</title>
<style>body{margin:0;font:16px/1.55 system-ui;background:#f3f7f6;color:#203330}main{max-width:1100px;margin:auto;padding:24px}h1,h2{font-weight:550}p{max-width:800px}.controls{display:flex;gap:16px;flex-wrap:wrap;padding:16px;background:#dfece8;border-radius:16px;position:sticky;top:0}select{font:inherit;padding:8px;border:1px solid #8da6a0;border-radius:8px;background:white;color:inherit}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:24px;margin-top:24px}figure{margin:0}img{width:100%;height:auto;border-radius:8px;border:1px solid #bbcfc9}figcaption{margin:8px 0 24px}code{background:#e3ece9;padding:2px 5px;border-radius:4px}a{color:#075f52}</style>
<main><h1>Northstar Studio</h1><p>A separate desktop app consuming the library through its public API. These are actual renderer captures of navigation, a custom surface, native iced controls and nested dialogs.</p>
<div class="controls"><label>Appearance <select id="theme"><option value="light">Light</option><option value="dark">Dark</option></select></label><label>Window / scale <select id="size"><option value="1000-1.25">1000px · 125%</option><option value="420-1">420px · 100%</option><option value="1000-2">1000px · 200%</option></select></label></div>
<div class="grid"><figure><a><img data-scene="overview" alt="Studio overview with metrics and progress"></a><figcaption>Overview · Material cards and progress with an ordinary iced action.</figcaption></figure><figure><a><img data-scene="preferences" alt="Preferences on a custom gradient surface"></a><figcaption>Preferences · outlined fields preserve a custom parent surface.</figcaption></figure><figure><a><img data-scene="editor" alt="Workspace editor containing Material and native iced controls"></a><figcaption>Editor · Material fields/select, native text input and a separately themed pick list.</figcaption></figure><figure><a><img data-scene="discard" alt="Nested discard confirmation"></a><figcaption>Nested confirmation · keep editing restores the underlying native field focus.</figcaption></figure></div>
<h2>Try the app</h2><p>From the repository, run <code>cargo run --locked --manifest-path consumers/desktop/Cargo.toml</code>. Use Edit workspace, change a field, then cancel to try the nested dialog. Preferences contains the appearance and reduced-motion controls.</p>
<p>All changes stay in memory. These captures use reduced motion for stable inspection. The runnable app animates normally. This review does not establish native Windows/Linux or screen-reader support.</p></main>
<script>const theme=document.querySelector('#theme'),size=document.querySelector('#size');function show(){for(const image of document.querySelectorAll('[data-scene]')){const path=`${theme.value}-${size.value}-${image.dataset.scene}.png`;image.src=path;image.parentElement.href=path;}}theme.addEventListener('change',show);size.addEventListener('change',show);show();</script></html>''', encoding="utf-8")
print(root / "index.html")

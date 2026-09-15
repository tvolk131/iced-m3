"""Build a dependency-free, offline review page for captured visual test frames."""
from pathlib import Path
import html
import json

root = Path(__file__).resolve().parents[2] / "target" / "visual-report"
groups = {}
for actual in sorted(root.glob("**/actual.png")):
    group = actual.parent.parent.relative_to(root).as_posix()
    groups.setdefault(group, []).append({
        "label": actual.parent.name,
        "src": actual.relative_to(root).as_posix(),
        "comparison": (actual.parent / "index.html").relative_to(root).as_posix(),
    })

sections = []
for name, frames in groups.items():
    label = html.escape(name)
    data = html.escape(json.dumps(frames), quote=True)
    sections.append(f'''<section class="case" data-name="{label}" data-frames="{data}">
      <h2>{label}</h2><p><output>{html.escape(frames[0]['label'])}</output></p>
      <input type="range" min="0" max="{len(frames)-1}" value="0" aria-label="Saved frame for {label}">
      <span>{len(frames)} saved frames</span>
      <p><a href="{frames[0]['comparison']}">Expected / actual / difference</a></p>
      <img loading="lazy" src="{frames[0]['src']}" alt="{label}">
      <details><summary>All frames</summary><div class="strip">''' + "".join(
        f'<figure><a href="{f["comparison"]}"><img loading="lazy" src="{f["src"]}" alt="{html.escape(f["label"])}"></a><figcaption>{html.escape(f["label"])}</figcaption></figure>'
        for f in frames
      ) + '</div></details></section>')

root.mkdir(parents=True, exist_ok=True)
(root / "index.html").write_text('''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Material visual tests</title>
<style>body{font:16px/1.5 system-ui;margin:0;background:#f6f5fa;color:#242129}header,main{max-width:1180px;margin:auto;padding:24px}header{position:sticky;top:0;background:#f6f5faed;z-index:1}h1{margin:0}input[type=search]{font:inherit;padding:10px;width:min(440px,90%)}.case{background:white;border:1px solid #d8d4df;border-radius:12px;padding:20px;margin:20px 0}.case>img{max-width:100%;max-height:600px;object-fit:contain;object-position:left top;display:block;border:1px solid #ddd}input[type=range]{width:min(480px,80%)}.strip{display:flex;gap:12px;overflow:auto;padding:12px 0}figure{margin:0;flex:0 0 280px}figure img{width:280px;border:1px solid #ddd}figcaption{font-size:12px}output{font-family:monospace}a{color:#634499}details{margin-top:16px}[hidden]{display:none}</style>
<header><h1>Material visual tests</h1><p>Scrub precisely timed reference frames. Each capture uses the real renderer; the slider selects saved frames without interpolating between them.</p>
<input type="search" id="filter" placeholder="Filter: switch, snackbar, gallery…" aria-label="Filter components"></header><main>'''
    + f'<p>{len(groups)} groups · {sum(map(len, groups.values()))} captured frames</p>'
    + ''.join(sections) + '''</main><script>
const filter=document.querySelector('#filter');
filter.value=new URLSearchParams(location.search).get('filter')||'';
const sections=[...document.querySelectorAll('.case')];
function applyFilter(){const query=filter.value.toLowerCase().trim();sections.forEach(s=>s.hidden=!s.dataset.name.includes(query));}
function showFrame(section){const frames=JSON.parse(section.dataset.frames);const f=frames[Number(section.querySelector('input').value)];section.querySelector('output').textContent=f.label;section.querySelector(':scope > img').src=f.src;section.querySelector(':scope > p > a').href=f.comparison;}
filter.addEventListener('input',applyFilter);
sections.forEach(section=>section.querySelector('input').addEventListener('input',()=>showFrame(section)));
function restoreView(){applyFilter();sections.forEach(showFrame);}
window.addEventListener('pageshow',restoreView);
restoreView();
</script></html>''', encoding='utf-8')
print(root / "index.html")

# Focused motion player. Before/after panels appear when the local backup exists.
import runpy
runpy.run_path(str(Path(__file__).with_name("baseline-preview.py")))

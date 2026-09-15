"""Build an offline review of shipped backgrounds and test-only renderer probes."""
from pathlib import Path
import hashlib
import html
import shutil

repo = Path(__file__).resolve().parents[2]
target = repo / "target"
root = target / "visual-report" / "composition-review"
root.mkdir(parents=True, exist_ok=True)
references = repo / "tests" / "visual" / "references"
backup = target / "composition-foundations-before" / "references"


def picture(source, label, name):
    if not source.is_file():
        return ""
    destination = root / "images" / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    return (f'<figure><a href="images/{html.escape(name, quote=True)}">'
            f'<img loading="lazy" src="images/{html.escape(name, quote=True)}" '
            f'alt="{html.escape(label, quote=True)}"></a>'
            f'<figcaption>{html.escape(label)}</figcaption></figure>')


sections = ['<h2>Shipped: tabs and lists</h2><p>Transparent containers preserve '
            'the striped parent. Explicit fills remain available. Held and moving '
            'selection frames are real renderer captures.</p><div class="grid">']
for source in sorted((references / "composition-foundations").glob("*.png")):
    sections.append(picture(source, source.stem, "references/" + source.name))
sections.append('</div><h2>Gallery changes</h2>')
if backup.is_dir():
    for source in sorted(references.rglob("*.png")):
        path = source.relative_to(references)
        old = backup / path
        if not old.is_file() or hashlib.sha256(old.read_bytes()).digest() == hashlib.sha256(source.read_bytes()).digest():
            continue
        sections.append(f'<h3>{html.escape(path.as_posix())}</h3><div class="pair">')
        sections.append(picture(old, "Before", "before/" + path.as_posix()))
        sections.append(picture(source, "After", "after/" + path.as_posix()))
        sections.append('</div>')
else:
    sections.append('<p>The local before-change backup is unavailable. The checked-in reference frames above remain reviewable.</p>')

sections.append('<h2>Experiments only: child clipping and group opacity</h2>'
                '<p>These paths are not enabled in the library or gallery. '
                'Direct and Bands use full opacity; Offscreen applies 65% group '
                'opacity. Bands approximate rounded edges by replaying 57 clips. '
                'Offscreen captures on the CPU, then uploads a PNG inside an SVG.</p>')
probe = target / "compositing-probe"
for backend in ["tiny-skia", "wgpu"]:
    sections.append(f'<h3>{html.escape(backend)} · 560 × 520 at 2x</h3><div class="grid">')
    for mode in ["Direct", "Bands", "Offscreen"]:
        name = f"{backend}-profile-{mode}-560.png"
        sections.append(picture(probe / name, mode, "probe/" + name))
    sections.append('</div>')
    sections.append('<h3>Translucent children at 100% group opacity</h3>'
                    '<p>CPU isolation changes the blending result on Metal. '
                    'Compare the purple overlap, not only the rounded corners.</p><div class="pair">')
    for mode in ["native", "offscreen"]:
        name = f"{backend}-translucent-{mode}.png"
        sections.append(picture(probe / name, f"{backend}: {mode}", "probe/" + name))
    sections.append('</div>')
    log = target / f"compositing-probe-{'gpu' if backend == 'wgpu' else 'software'}-profile.log"
    if log.is_file():
        rows = [line for line in log.read_text().splitlines()
                if line.startswith(("Direct ", "Bands ", "Offscreen ", "Translucent child"))]
        sections.append('<details><summary>Measured release capture costs</summary><pre>'
                        + html.escape("\n".join(rows)) + '</pre></details>')

(root / "index.html").write_text('''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Composition review</title>
<style>body{font:16px/1.5 system-ui;background:#f6f5fa;color:#27242e;margin:0;padding:32px}main{max-width:1440px;margin:auto}h1{margin-bottom:8px}h2{margin-top:48px}h3{margin-top:28px}p{max-width:850px}.grid,.pair{display:grid;gap:20px;align-items:start}.grid{grid-template-columns:repeat(auto-fit,minmax(320px,1fr))}.pair{grid-template-columns:repeat(2,minmax(0,1fr))}figure{margin:0;background:white;border:1px solid #dad4e5;padding:12px;border-radius:12px}img{width:100%;height:auto;display:block}figcaption{font-size:13px;overflow-wrap:anywhere;padding-top:8px}pre{overflow:auto;padding:16px;background:white}a{color:#624390}@media(max-width:700px){body{padding:16px}.pair{grid-template-columns:1fr}}</style>
<main><h1>Composition review</h1><p>Tabs and lists now preserve their parent background. The renderer experiments below establish the remaining boundary for arbitrary-child clipping and fades.</p>
<p><a href="../?filter=composition-foundations">Timed expected / actual / difference frames</a></p>'''
    + "".join(sections) + '</main></html>', encoding="utf-8")
print(root / "index.html")

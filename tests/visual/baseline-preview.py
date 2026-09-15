"""A focused, offline player for the baseline completion reference captures."""
from pathlib import Path
import html
import json
import shutil

base = Path(__file__).resolve().parents[2]
root = base / 'target/visual-report'
page = root / 'baseline-completion'
page.mkdir(parents=True, exist_ok=True)
sequences = [
    ('Range labels', 'Both labels separate as keyboard focus reveals them.', 'baseline-range-close', 'focus', [0,50,100], 3),
    ('Checkbox selection', 'Independent opacity, scale and check-stroke timing.', 'baseline-checkbox', 'select', [0,25,50,100,175,350], 3),
    ('Checkbox clearing', 'The fill disappears while the hover circle remains.', 'baseline-checkbox', 'clear', [0,25,50,100,150], 3),
    ('Filter and avatar chips', 'The filter icon transitions; the input avatar stays in place.', 'baseline-chips', 'select', [0,25,75,150,200], 3),
    ('Circular loading → measured value', 'The loading arc closes, then the measured value follows a spring.', 'baseline-progress-circular', 'handoff', [0,100,200,333,500,1000,1550,1650,1750,2050,3000,4000], 4),
    ('Linear loading → measured value', 'The disjoint cycle finishes before measured progress takes over.', 'baseline-progress-linear', 'handoff', [0,100,200,333,500,1000,1550,1650,1750,2050,3000,4000], 4),
    ('Dialog opening', 'A rounded growing panel, downward arrival, and separate surface/content/scrim fades.', 'baseline-dialog', 'open', [0,25,50,100,150,250,500], 3),
    ('Dialog closing', 'The content fades first, then the surface and shadow disappear.', 'baseline-dialog', 'close', [0,50,100,149,150], 3),
    ('Menu opening', 'The rounded menu surface grows without scaling its text.', 'baseline-menu', 'open', [0,25,50,100,250,500], 3),
    ('Menu closing', 'The surface contracts to 35% and fades through the final 50ms.', 'baseline-menu', 'close', [0,50,100,149,150], 3),
]
sections = []
for title, description, group, phase, times, digits in sequences:
    frames = []
    for ms in times:
        paths = {mode: f'../{group}/{mode}/{phase}-{ms:0{digits}}ms/actual.png' for mode in ['light','dark']}
        if all((page / path).is_file() for path in paths.values()):
            frames.append({'ms': ms, **paths})
    if not frames:
        continue
    data = html.escape(json.dumps(frames), quote=True)
    sections.append(f'''<section class="sequence" data-frames="{data}"><h2>{html.escape(title)}</h2><p>{html.escape(description)}</p>
<div class="controls"><button class="play" type="button">Play</button><input type="range" min="0" max="{frames[-1]['ms']}" value="{frames[-1]['ms']}" aria-label="Time for {html.escape(title)}"><output>{frames[-1]['ms']} ms</output></div>
<img class="frame" src="{frames[-1]['light']}" alt="{html.escape(title)}"></section>''')

before = base / 'target/baseline-completion-before/references'
after = base / 'tests/visual/references'
pairs = [
    ('Date header', 'A 120px single-date header and a full-width divider.', 'picker-date-single/light/00-default.png'),
    ('Time header', 'The title begins at 16px and the time display at 44px.', 'picker-time-12h-Hour/light/00-default.png'),
    ('Held range slider', 'Holding a handle now displays both values.', 'range-slider-coincident/light/02-held-150ms.png'),
    ('Dialog at 100ms', 'The new arrival starts slightly above its destination.', 'polish-dialog/light/01-open-100ms.png'),
]
comparisons = []
for i, (title, description, path) in enumerate(pairs):
    if not (before/path).is_file() or not (after/path).is_file():
        continue
    shutil.copyfile(before/path,page/f'before-{i}.png')
    shutil.copyfile(after/path,page/f'after-{i}.png')
    comparisons.append(f'''<section><h2>{title}</h2><p>{description}</p><div class="pair"><figure><figcaption>Previous library</figcaption><img src="before-{i}.png" alt="Previous {title}"></figure><figure><figcaption>Updated library</figcaption><img src="after-{i}.png" alt="Updated {title}"></figure></div></section>''')

(page/'index.html').write_text('''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Desktop motion and layout</title>
<style>:root{color-scheme:light dark}body{font:16px/1.5 system-ui;margin:0;background:#f8f7fc;color:#292330}main{max-width:1100px;margin:auto;padding:28px}h1{margin:0;font-size:32px}h2{font-size:22px;margin:0}p{max-width:84ch}section{background:white;border:1px solid #d9d3e1;border-radius:16px;padding:20px;margin:24px 0}.controls{display:flex;align-items:center;gap:16px;margin:16px 0}.controls input{flex:1;max-width:480px;accent-color:#68548e}button,select{font:inherit;padding:8px 16px;border:1px solid #a499b5;border-radius:20px;background:#eee8f5;color:#32234b}output{font-variant-numeric:tabular-nums;min-width:84px}img{display:block;max-width:100%;max-height:510px;object-fit:contain;object-position:top left}.pair{display:grid;grid-template-columns:1fr 1fr;gap:20px}figure{margin:0;min-width:0}figcaption{font-weight:600;margin-bottom:8px}a{color:#624499}.note{font-size:14px;color:#665d70}@media(max-width:600px){main{padding:16px}.pair{grid-template-columns:1fr}.controls{gap:8px}}@media(prefers-color-scheme:dark){body{background:#151219;color:#e9e2ef}section{background:#221e28;border-color:#4c4359}a{color:#d3bafa}.note{color:#c3b9cd}}</style>
<main><h1>Desktop motion and layout</h1><p>Readable range values, refined selection feedback, loading completion, rounded surface motion, and picker spacing.</p><p><a href="/?filter=gallery-baseline">Gallery preview</a> · <a href="/?filter=baseline-">All timed comparisons</a></p>
<p class="note">These are saved frames from iced's actual renderer. Playback holds each captured frame until the next timestamp; it does not interpolate or recreate the animation in the browser.</p>
<label>Component appearance <select id="theme"><option value="light">Light</option><option value="dark">Dark</option></select></label>'''+''.join(sections)+
('<h2>Before and after</h2><p class="note">These compare two versions of this library, not screenshots of Google components.</p>'+''.join(comparisons) if comparisons else '')+'''
<p class="note">Individual menu-row staggering and the later dialog action fade remain separate work. The broader platform and Expressive boundaries are documented in the repository.</p>
</main><script>
const appearance=document.querySelector('#theme');
const entries=[...document.querySelectorAll('.sequence')].map(section=>({section,frames:JSON.parse(section.dataset.frames),input:section.querySelector('input'),button:section.querySelector('button'),output:section.querySelector('output'),image:section.querySelector('.frame'),playing:false,start:0}));
function show(e){const time=Number(e.input.value);const frame=[...e.frames].reverse().find(f=>f.ms<=time)||e.frames[0];e.image.src=frame[appearance.value];e.output.textContent=frame.ms+' ms';}
function stop(e){e.playing=false;e.button.textContent='Play';}
function tick(now){let active=false;for(const e of entries){if(!e.playing)continue;const elapsed=now-e.start;e.input.value=Math.min(elapsed,Number(e.input.max));show(e);if(elapsed>=Number(e.input.max)){stop(e)}else active=true;}if(active)requestAnimationFrame(tick);}
for(const e of entries){e.input.addEventListener('input',()=>{stop(e);show(e)});e.button.addEventListener('click',()=>{if(e.playing){stop(e);return;}entries.forEach(stop);e.playing=true;e.start=performance.now();e.button.textContent='Pause';e.input.value=0;show(e);requestAnimationFrame(tick)});show(e);}
appearance.addEventListener('change',()=>entries.forEach(show));
document.addEventListener('visibilitychange',()=>{if(document.hidden)entries.forEach(stop)});
</script></html>''',encoding='utf-8')
print(page/'index.html')

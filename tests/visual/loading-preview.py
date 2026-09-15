"""Offline player for real, precisely timed Rust renderer captures."""
from pathlib import Path
import shutil

repo = Path(__file__).resolve().parents[2]
root = repo / "target/visual-report/loading-progress-motion"
root.mkdir(parents=True, exist_ok=True)
shape_root = root / "shapes"
shape_root.mkdir(exist_ok=True)
names = ["soft-burst", "cookie-9", "pentagon", "pill", "sunny", "cookie-4", "oval"]
for name in names:
    shutil.copyfile(repo / f"assets/loading/reference/{name}.svg", shape_root / f"{name}.svg")
shapes = "".join(f'<figure><img src="shapes/{name}.svg" alt="{name}"><figcaption>{name.replace("-", " ")}</figcaption></figure>' for name in names)
(root / "index.html").write_text('''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Loading with expression</title>
<style>body{margin:0;background:#f8f5fc;color:#26212c;font:16px/1.5 system-ui}main{max-width:1060px;margin:auto;padding:28px}h1{font-weight:500;margin-bottom:8px}p{max-width:850px}.controls{display:flex;gap:16px;align-items:center;flex-wrap:wrap;background:#eee7f5;padding:16px;border-radius:16px;position:sticky;top:0;z-index:1}button,select{font:inherit;padding:8px 16px;border:1px solid #b9abca;border-radius:20px;background:white;color:inherit}button{cursor:pointer}input{flex:1;min-width:140px;accent-color:#67508f}output{font-variant-numeric:tabular-nums;min-width:75px}.screen{display:flex;justify-content:center;margin:24px 0}.screen img{max-height:720px;max-width:100%;width:auto;border:1px solid #d2c7dd;border-radius:8px}.shapes{display:flex;gap:12px;flex-wrap:wrap}.shapes figure{margin:0;background:white;border:1px solid #ded6e7;border-radius:12px;padding:10px;text-align:center;min-width:100px}.shapes img{width:72px;height:72px}figcaption{font-size:13px}a{color:#63478b}details{padding:12px 0}small{color:#645b6d}</style>
<main><h1>Loading with expression</h1><p>Canonical rounded shapes and wavy progress, captured from the Rust renderer at 30 frames per second. The striped background makes transparency and track gaps visible.</p>
<div class="controls"><button id="play" type="button">Play</button><select id="theme" aria-label="Theme"><option value="light">Light</option><option value="dark">Dark</option></select><input id="frame" type="range" min="0" max="180" value="0" aria-label="Captured animation frame"><output id="time">0.00 s</output></div>
<div class="screen"><img id="screen" src="frames/light-000.png" alt="Loading indicators and wavy linear/circular progress"></div>
<p><small>Playback is a six-second clip and stops at the end. It selects saved frames without inventing intermediate images. The second measured ring has optional wave travel enabled.</small></p>
<h2>The seven source shapes</h2><p>These outlines come from the pinned Material/AndroidX shape code. The loader morphs their matched curves and adds spring-driven rotation.</p><div class="shapes">'''+shapes+'''</div>
<h2>Try the controls</h2><p>In the rebuilt gallery: Workspace → Components → Loading with expression. Switch between loading and measured progress, pause motion, enable traveling waves, and move the progress slider.</p>
<p><a href="../?filter=loading-progress">Timed golden frames and completion handoffs</a> · <a href="../?filter=gallery-loading-progress">Narrow and wide gallery captures</a></p>
<details><summary>Reference and implementation</summary><p>The loader uses the seven canonical morphs with a 650ms spring cadence. Wavy linear loading uses the 1.8s disjoint profile; circular loading uses the 6s single-section retreat profile. Measured waves flatten near the endpoints. Drawing uses cached SVG paths on upstream iced.</p></details></main>
<script>
const control=document.querySelector('#frame'),theme=document.querySelector('#theme'),screen=document.querySelector('#screen'),button=document.querySelector('#play'),time=document.querySelector('#time');
let playing=false,origin=0,start=0;
function source(mode,index){return `frames/${mode}-${String(index).padStart(3,'0')}.png`;}
function show(){screen.src=source(theme.value,+control.value);time.value=(Number(control.value)/30).toFixed(2)+' s';}
function stop(){playing=false;button.textContent='Play';}
function tick(now){if(!playing)return;const frame=Math.min(180,start+Math.floor((now-origin)*30/1000));control.value=frame;show();if(frame===180)stop();else requestAnimationFrame(tick);}
button.addEventListener('click',()=>{if(playing){stop();return;}if(+control.value===180)control.value=0;start=+control.value;origin=performance.now();playing=true;button.textContent='Pause';requestAnimationFrame(tick);});
control.addEventListener('input',()=>{stop();show();});theme.addEventListener('change',show);
document.addEventListener('visibilitychange',()=>{if(document.hidden)stop();});
// Warm the small capture set so disk/network latency doesn't drive animation.
for(const mode of ['light','dark'])for(let i=0;i<=180;i++){const image=new Image();image.src=source(mode,i);}
show();
</script></html>''', encoding="utf-8")
print(root / "index.html")

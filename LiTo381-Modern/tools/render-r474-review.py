"""Review UI from genuine Rust CPU outputs; no game state/runtime claims."""
import json,html,hashlib
from pathlib import Path
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r474-offline-map-flow'
m=json.loads((L/'r474-mode-map-binding.json').read_text(encoding='utf-8'));d=json.loads((L/'r474-offline-map-details.json').read_text(encoding='utf-8'))
for image in d['images']:
 p=Path(image['path']);assert hashlib.sha256(p.read_bytes()).hexdigest().upper()==image['sha256']
options=[]
for row in m['rows']:
 c=row['original_table'];selected=c['mode']==1 and c['map_id']==564
 options.append('<option value="%s" %s>%s · %s%s</option>'%(c['map_id'],'' if selected else 'disabled',html.escape(c['name']),c['map_id'],'' if selected else '（尚未可預覽）'))
page='''<!doctype html><html lang="zh-Hant"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>LiTo Modern · 離線場地預覽</title><style>
*{box-sizing:border-box}body{margin:0;background:#101916;color:#e8eee7;font:16px system-ui,"Microsoft JhengHei",sans-serif}main{max-width:1080px;margin:44px auto;padding:0 28px}small,.muted{color:#a5b3a7}h1{font-size:34px;margin:12px 0}h2{font-size:24px}header{display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid #344237;padding-bottom:22px}.brand{letter-spacing:3px;color:#d4b474}section{margin-top:28px;background:#1a2720;border:1px solid #394a3d;border-radius:14px;padding:26px}select{width:100%;max-width:520px;background:#101b15;border:1px solid #61775d;color:#edf2e9;padding:14px;border-radius:8px;font:inherit}button{background:#d3b575;color:#152017;border:0;border-radius:8px;padding:12px 22px;font:600 16px system-ui;cursor:pointer}button.secondary{background:#334537;color:#eef1ea}button:hover{filter:brightness(1.12)}.actions{display:flex;gap:12px;margin-top:20px;flex-wrap:wrap}.map{width:100%;max-width:800px;display:block;margin:18px auto;border:1px solid #4e6150;image-rendering:auto}p{line-height:1.7}.pill{font-size:13px;border:1px solid #55745a;border-radius:30px;padding:6px 12px;color:#bdd0b9}[hidden]{display:none!important}footer{margin:22px 0;color:#a5b3a7;font-size:14px}.status{font-size:14px;color:#c3d1c0}kbd{border:1px solid #61775d;border-radius:4px;padding:2px 7px}
</style><main><header><div><div class="brand">LITO MODERN</div><h1>離線場地預覽</h1><small>原版地圖與場地名稱 · CPU 畫面檢視</small></div><span class="pill">離線</span></header>
<section id="menu"><small>01 / 選擇場地</small><h2>Death Match</h2><p class="muted">選擇 Battle Ground，查看場地的原版圖像。</p><label for="map">場地</label><p><select id="map">OPTIONS</select></p><div class="actions"><button id="enter">進入預覽 →</button></div><p class="status" id="menuStatus">目前可預覽：Battle Ground · 564</p></section>
<section id="preview" hidden><small>02 / 場地預覽</small><h2>Battle Ground <small>564</small></h2><img class="map" id="image" src="01-entry-battle-ground.png" alt="Battle Ground 原版地圖的 Rust CPU 渲染"><div class="actions"><button id="pan">查看右下方 ↘</button><button id="reset" class="secondary">重置視角</button><button id="back" class="secondary">← 返回場地選單</button></div><p class="status" id="camera">視角：中央</p><p class="muted">使用 <kbd>→</kbd> 查看右下方、<kbd>←</kbd> 重置、<kbd>Esc</kbd> 返回。</p></section>
<footer>這是離線地圖預覽。可檢視兩個實際渲染視角；尚未包含角色、戰鬥或連線。此頁展示 Rust 產物，互動式 Rust 工具另附。</footer></main><script>
const menu=document.getElementById('menu'),preview=document.getElementById('preview'),image=document.getElementById('image'),camera=document.getElementById('camera');let visits=0;
function reset(){image.src=visits>1?'03-reentry-battle-ground.png':'01-entry-battle-ground.png';camera.textContent='視角：中央'}
function pan(){if(!preview.hidden){image.src='02-pan-battle-ground.png';camera.textContent='視角：向右 240、向下 120'}}
function back(){preview.hidden=true;menu.hidden=false;document.getElementById('menuStatus').textContent='已返回選單 · Battle Ground 仍保持選取';document.getElementById('enter').focus()}
document.getElementById('enter').onclick=()=>{visits++;menu.hidden=true;preview.hidden=false;reset();document.getElementById('back').focus()};document.getElementById('pan').onclick=pan;document.getElementById('reset').onclick=reset;document.getElementById('back').onclick=back;document.addEventListener('keydown',e=>{if(preview.hidden)return;if(e.key==='Escape')back();else if(e.key==='ArrowRight')pan();else if(e.key==='ArrowLeft')reset();else return;e.preventDefault()});
</script></html>'''.replace('OPTIONS','\n'.join(options))
path=O/'offline-map-preview.html';assert not path.exists();path.write_text(page,encoding='utf-8');print(path)

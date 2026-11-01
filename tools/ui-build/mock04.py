"""Build the 0.4 preview artifact from app/ui/index.html: sample data + proposed features."""
import sys, re
src, dst = sys.argv[1], sys.argv[2]
h = open(src, encoding='utf-8').read()

def rep(old, new, count=1):
    global h
    n = h.count(old)
    assert n == count, f"expected {count} of {old[:90]!r}, found {n}"
    h = h.replace(old, new)

def between(start, end, new):
    global h
    a = h.index(start); b = h.index(end, a)
    h = h[:a] + new + h[b:]

# ---- artifact skeleton: no document tags of our own ----
h = re.sub(r'<!doctype html>\s*<html[^>]*>\s*<head>\s*<meta charset="utf-8">\s*<meta name="viewport"[^>]*>\s*', '', h, count=1)
rep('<meta name="theme-color" content="#2a201a">\n<link rel="manifest" href="/manifest.json">\n<link rel="icon" href="/icon.svg">\n<title>Dev Room</title>',
    '<title>Dev Room 0.4 Preview</title>\n<meta name="claude:viewport-fit-sides" content="cover">')
rep('</style>\n</head>\n<body>\n', '</style>\n')
rep('</script>\n</body>\n</html>\n', '</script>\n')

# ---- styles for the new pieces ----
rep('/* app build: no device preview, no in-app reply yet */', r'''/* 0.4 preview */
.fxbar{display:flex;flex-wrap:wrap;gap:6px 8px;align-items:center}
.fxbar label{display:inline-flex;align-items:center;gap:5px;font-size:12.5px;padding:3px 9px;border:1px solid var(--line);border-radius:999px;background:var(--bg);cursor:pointer}
.fxbar input{accent-color:var(--accent)}
.mocktag{font-family:var(--f-mono);font-size:10.5px;font-weight:700;letter-spacing:.06em;text-transform:uppercase;color:var(--panel);background:var(--accent);padding:2px 7px;border-radius:4px}
.en{width:calc(26px * var(--bs,1.3));height:3px;border-radius:2px;background:rgba(43,33,27,.45);overflow:hidden;margin-top:1px}
.en i{display:block;height:100%}
.react{position:absolute;left:50%;bottom:100%;transform:translateX(-50%);font-size:calc(13px * var(--bs,1.3));animation:floatUp 2.4s ease-out forwards;pointer-events:none}
@keyframes floatUp{0%{opacity:0;transform:translate(-50%,6px) scale(.6)}15%{opacity:1;transform:translate(-50%,0) scale(1.1)}100%{opacity:0;transform:translate(-50%,-34px) scale(1)}}
@media (prefers-reduced-motion:reduce){.react{animation-duration:.01s}}
.tl8{grid-column:1/-1;display:flex;height:6px;border-radius:3px;overflow:hidden;background:var(--chip)}
.tl8 b{display:block;height:100%}
.mood{font-size:13px;margin-left:4px}
/* app build: no device preview, no in-app reply yet */''')

# ---- feature switches + helpers ----
rep("let TOKEN='';", r"""const FX={icons:true,effects:true,mood:true,timeline:true,walk:true};
const FX_LABEL={icons:'Status icon sa ulo',effects:'Effects at reactions',mood:'Energy at mood',timeline:'Timeline ng araw',walk:'Lakad na may direksyon'};
const ICON_IDLE={coffee:'☕',eat:'🍜',tv:'📺',billiards:'🎱',chess:'♟️',darts:'🎯',phone:'📱',stretch:'🙆',chismis:'💬',cr:'🚻',wander:'🚶',sleep:'💤',doze:'💤',desk:'🪑',check:'👀',rounds:'📋'};
function iconFor(e){
  if(e===guard)return guard.state==='doze'?'💤':'📱';
  if(e===boss)return{meet:'🗣️',watch:'👀',break:'☕'}[boss.mode]||'💼';
  if(e.npc)return'';
  if(e.path.length)return'🚶';
  return{working:'⌨️',needs:'✋',done:'✅',failed:'⚠️'}[e.status]||ICON_IDLE[e.idle]||'☕';
}
function energy(s){
  const ctx=s.ctx&&s.ctx.max_tokens?s.ctx.used_tokens/s.ctx.max_tokens:.3;
  const work=s.status==='working'?Math.min((now()-s.since)/MIN/180,.35):0;
  const rest=s.status==='idle'?Math.min(idleMins(s)/60,.3):0;
  return Math.max(.05,Math.min(1,1-ctx*.6-work+rest));
}
const moodOf=s=>s.status==='failed'?'😣':s.status==='needs'?'🤔':s.status==='done'?'😊':energy(s)<.3?'😩':s.status==='working'?'🧐':s.idle==='sleep'||s.idle==='doze'?'😴':'🙂';
function react(e,emo){if(!FX.effects)return;const n=nodes.get(e);if(!n)return;const r=document.createElement('span');r.className='react';r.textContent=emo;n.appendChild(r);setTimeout(()=>r.remove(),2500);}
const REACT={working:['💡','⌨️','🧐'],needs:['❓','🙋'],tv:['😂','😮'],billiards:['🎉','😤'],chess:['🤔','😏'],darts:['🎯','😅'],chismis:['😂','😮','🤭'],coffee:['☕'],eat:['😋'],phone:['😂','💬'],stretch:['😌']};
const TLC={working:'var(--work)',needs:'var(--need)',idle:'var(--idle)',done:'var(--done)',failed:'var(--fail)',sleep:'var(--sleep)'};
function seedTimeline(s){
  // sample history for the morning, ending with the real current state
  let x=hashStr(s.sid),t=now()-8*60*MIN;const tl=[];
  while(t<s.since-5*MIN){x=(x*1103515245+12345)>>>0;const st=['working','working','idle','working','needs','idle','working'][x%7];tl.push({st,from:t});t+=(15+x%70)*MIN;}
  tl.push({st:s.status==='idle'&&idleMins(s)>30?'sleep':s.status,from:Math.min(s.since,now())});s.tl=tl;
}
function tlHTML(s){
  if(!s.tl)seedTimeline(s);
  const start=now()-8*60*MIN,span=now()-start;
  return'<span class="tl8" title="Huling 8 oras">'+s.tl.map((g,i)=>{const a=Math.max(g.from,start),b=i+1<s.tl.length?s.tl[i+1].from:now();if(b<=a)return'';return`<b style="width:${((b-a)/span*100).toFixed(2)}%;background:${TLC[g.st]||TLC.idle}"></b>`;}).join('')+'</span>';
}
let TOKEN='';""")
rep("  const prev=s.status;s.status=st;s.since=s.at||now();",
    "  const prev=s.status;s.status=st;s.since=s.at||now();\n  if(s.tl)s.tl.push({st,from:now()});")

# ---- connection bar becomes the preview bar; data comes from a scripted sample team ----
between("function setConn(){", "async function poll(){", r"""let connBuilt=false;
function setConn(){
  const el=$('conn');if(!el||connBuilt)return;connBuilt=true;el.dataset.mode='live';
  el.innerHTML=`<span class="mocktag">Preview 0.4</span><span class="cm">Sample na team ito na may kunwaring events kada 20 segundo. I-off ang isang feature para makita ang pagkakaiba.</span><span class="fxbar">${Object.keys(FX).map(k=>`<label><input type="checkbox" id="fx-${k}" data-fx="${k}" checked>${FX_LABEL[k]}</label>`).join('')}</span>`;
  el.addEventListener('change',e=>{const k=e.target.dataset.fx;if(k){FX[k]=e.target.checked;render();}});
}
""")
between("async function poll(){", "let uiBuild=null;", r"""const MM=MIN,T0=Date.now();
const cu=(u)=>({context_usage:{used_tokens:u,max_tokens:1000000}});
const REC={
 L:{id:'L',title:'LOGIC_SIM_PROJECT',status_bucket:'SESSION_STATUS_BUCKET_COMPLETED',updated_at:T0-12*MM,cwd:'C:\\Users\\Ronan\\icms-master-tool',post_turn_summary:{recent_action:'Na-review ko na ang PR #88.'},external_metadata:cu(480000)},
 A:{id:'A',title:'CODE FIXER: LOGIC, ENGINE, PLANT MODEL',status_bucket:'SESSION_STATUS_BUCKET_WORKING',updated_at:T0-70*MM,short:'Nagko-code.',cwd:'C:\\Users\\Ronan\\icms-master-tool',post_turn_summary:{status_detail:'Ine-edit ang trip.ts.'},external_metadata:cu(780000)},
 B:{id:'B',title:'DOCS: TEAM STRUCTURE, NEW ACCOUNT, USER DECISIONS',status_bucket:'SESSION_STATUS_BUCKET_BLOCKED',updated_at:T0-3*MM,cwd:'C:\\Users\\Ronan\\icms-master-tool',post_turn_summary:{needs_action:'GO BETA ba para i-merge ko na ang PR #90?'},external_metadata:cu(105000)},
 C:{id:'C',title:'LIVE TESTER: COLD START-UP TO FULL LOAD (BETA.76)',status_bucket:'SESSION_STATUS_BUCKET_COMPLETED',updated_at:T0-1*MM,cwd:'C:\\Users\\Ronan\\icms-master-tool',post_turn_summary:{recent_action:'Tapos ang retest ng H3 hanggang H16.'},external_metadata:cu(102000)},
 D:{id:'D',title:'APP UI/UX: WHOLE APP LOOK AND USABILITY',status_bucket:'SESSION_STATUS_BUCKET_COMPLETED',updated_at:T0-18*MM,cwd:'C:\\Users\\Ronan\\icms-master-tool',post_turn_summary:{recent_action:'Tinanggap ng leader ang dock fixes.'},external_metadata:cu(735000)},
 E:{id:'E',title:'HMI: MOCKUP AND IMPLEMENTATION (ONE STYLE, HSYM)',status_bucket:'SESSION_STATUS_BUCKET_COMPLETED',updated_at:T0-95*MM,cwd:'C:\\Users\\Ronan\\icms-master-tool',post_turn_summary:{recent_action:'Itinigil ang mga background drafter.'},external_metadata:cu(365000)}};
const SW_='SESSION_STATUS_BUCKET_WORKING',SB_='SESSION_STATUS_BUCKET_BLOCKED',SI_='SESSION_STATUS_BUCKET_COMPLETED';
const SCRIPT=[
 ['C',{status_bucket:SW_,short:'Nagbabasa ng utos.',prompt_at:1,post_turn_summary:{status_detail:'Inutusan: retest H17 hanggang H20.'}}],
 ['B',{status_bucket:SW_,short:'Nagko-code.',post_turn_summary:{status_detail:'Mine-merge ang PR #90.'}}],
 ['A',{status_bucket:SI_,post_turn_summary:{recent_action:'Naayos ang trip logic at pumasa ang 42 tests.'}}],
 ['C',{status_bucket:SW_,short:'Nagpapatakbo.',post_turn_summary:{status_detail:'Pinapatakbo ang cold start-up test.'}}],
 ['B',{status_bucket:SI_,post_turn_summary:{recent_action:'Na-merge na ang PR #90.'}}],
 ['D',{status_bucket:SW_,short:'Nagko-code.',prompt_at:1,post_turn_summary:{status_detail:'Ine-edit ang Dock.tsx.'}}],
 ['C',{status_bucket:SB_,post_turn_summary:{needs_action:'Nag-trip ang H18. Uulitin ko ba ang test?'}}],
 ['L',{status_bucket:SW_,short:'Nagpaplano.',post_turn_summary:{status_detail:'Hinahati ang mga task para sa beta.77.'}}],
 ['C',{status_bucket:SW_,short:'Nagpapatakbo.',post_turn_summary:{status_detail:'Inuulit ang H18.'}}],
 ['D',{status_bucket:SI_,post_turn_summary:{recent_action:'Naayos ang spacing ng dock.'}}],
 ['L',{status_bucket:SI_,post_turn_summary:{recent_action:'Naipadala ang mga task sa team.'}}],
 ['A',{status_bucket:SW_,short:'Nagko-code.',prompt_at:1,post_turn_summary:{status_detail:'Ine-edit ang plant_model.ts.'}}],
 ['C',{status_bucket:SI_,post_turn_summary:{recent_action:'Pumasa na ang H17 hanggang H20.'}}],
 ['B',{status_bucket:SB_,post_turn_summary:{needs_action:'Ia-update ko ba ang handoff notes para sa beta.77?'}}]];
let step=0;
function push(){applyRecords(Object.values(REC).map(r=>JSON.parse(JSON.stringify(r))));}
function emit(){
  const [id,p]=SCRIPT[step++%SCRIPT.length];const r=REC[id];
  Object.assign(r,{status_bucket:p.status_bucket,short:p.short||'',updated_at:Date.now(),post_turn_summary:Object.assign({status_detail:'',needs_action:'',recent_action:''},p.post_turn_summary)});
  if(p.prompt_at)r.prompt_at=Date.now();
  push();
}
async function poll(){
  mode='live';isLocal=false;hooksOk=true;lastSync=Date.now();
  push();setConn();
  setInterval(()=>{if(!paused)emit();},20000);
}
""")

# ---- overlay: icon, energy bar ----
rep("    const tag=e.npc?e.name:e.short;n._t.title=e.npc?e.name:e.role;if(n._t.textContent!==tag)n._t.textContent=tag;",
    """    const ic=FX.icons?iconFor(e):'';const tag=(ic?ic+' ':'')+(e.npc?e.name:e.short);n._t.title=e.npc?e.name:e.role;if(n._t.textContent!==tag)n._t.textContent=tag;
    if(!e.npc&&FX.mood){if(!n._e){n._e=document.createElement('div');n._e.className='en';n._e.innerHTML='<i></i>';n.appendChild(n._e);}const en=energy(e);n._e.hidden=false;n._e.firstChild.style.width=Math.round(en*100)+'%';n._e.firstChild.style.background=en>.6?'#6f9a55':en>.3?'#d2a75a':'#c4573c';}else if(n._e)n._e.hidden=true;""")

# ---- dock: mood + timeline ----
rep("""<span class="nm" style="display:block">${esc(s.role)}</span>""", """<span class="nm" style="display:block">${esc(s.role)}${FX.mood?`<span class="mood" title="Energy ${Math.round(energy(s)*100)}%">${moodOf(s)}</span>`:''}</span>""")
rep("""<span class="dur">${dur(now()-s.since)}</span></span></button>`;""", """<span class="dur">${dur(now()-s.since)}</span></span>${FX.timeline?tlHTML(s):''}</button>`;""")

# ---- walking: face the direction of travel, swing arms ----
rep("        else{e.x+=dx/d*sp;e.y+=dy/d*sp;e.pose='walk';}}", "        else{e.x+=dx/d*sp;e.y+=dy/d*sp;e.pose='walk';const sx=dx-dy;if(Math.abs(sx)>.01)e.face=sx<0?-1:1;}}")
rep("  const x=Math.round(p.x-10*CS),y=Math.round(p.y-22*CS);\n  for(const[dx,dy] of [[-1,0],[1,0],[0,-1],[0,1]])ctx.drawImage(sil,x+dx*CS,y+dy*CS,20*CS,24*CS);\n  ctx.drawImage(spr,x,y,20*CS,24*CS);",
    "  const x=Math.round(p.x-10*CS),y=Math.round(p.y-22*CS);const flip=FX.walk&&e.face<0&&e.pose!=='sit'&&e.pose!=='sleep';\n  if(flip){ctx.save();ctx.translate(Math.round(p.x)*2,0);ctx.scale(-1,1);}\n  for(const[dx,dy] of [[-1,0],[1,0],[0,-1],[0,1]])ctx.drawImage(sil,x+dx*CS,y+dy*CS,20*CS,24*CS);\n  ctx.drawImage(spr,x,y,20*CS,24*CS);\n  if(flip)ctx.restore();")
rep("  else{px(x+4,y0+8+bob,1,4,shirt);px(x-5,y0+8+bob,1,4,shirt);}",
    "  else if(moving&&FX.walk){const sw=f?1:-1;px(x+4,y0+8+bob+sw,1,4,shirt);px(x-5,y0+8+bob-sw,1,4,shirt);}\n  else{px(x+4,y0+8+bob,1,4,shirt);px(x-5,y0+8+bob,1,4,shirt);}")

# ---- canvas effects: steam, billiard ball, darts, monitor glow ----
rep("  updateOverlay(t);\n  requestAnimationFrame(frame);", r"""  if(FX.effects)drawFx(t);
  updateOverlay(t);
  requestAnimationFrame(frame);""")
rep("/* ---------- tick: office life; statuses come from real data ---------- */", r"""function drawFx(t){
  const still=s=>!s.path.length;
  for(const e of PEOPLE()){
    const act=(e.idle||'');if(!still(e))continue;
    if(act==='coffee'){const p=isoP(e);for(let i=0;i<4;i++){const k=(t*.6+i/4+e.seed)%1;ctx.fillStyle=`rgba(239,230,216,${.55*(1-k)})`;ctx.fillRect(p.x+9+Math.sin(k*7+i)*2,p.y-30-k*16,2,2);}}
    if(e.status==='working'&&e.pose==='sit'){const p=isoP(e);ctx.fillStyle=`rgba(240,220,174,${.10+.05*Math.sin(t*3+e.seed)})`;ctx.beginPath();ctx.ellipse(p.x,p.y-26,14,10,0,0,Math.PI*2);ctx.fill();}
    if(act==='darts'){const ph=(t*.45+e.seed)%1;if(ph<.35){const a=isoP(e),b=iso(6.8,16.35,11);const k=ph/.35;ctx.fillStyle='#b03a2e';ctx.fillRect(a.x+(b.x-a.x)*k,a.y-30+(b.y-(a.y-30))*k-Math.sin(k*Math.PI)*8,3,2);}}
  }
  const players=sessions.filter(s=>s.idle==='billiards'&&still(s)).length;
  if(players){const sp=players>1?1:.5,u=4.15+1.9*(.5+.5*Math.sin(t*1.3*sp)),v=11.9+.85*(.5+.5*Math.sin(t*.9*sp+1));const p=iso(u,v,11.4);ctx.fillStyle='#efe6d8';ctx.beginPath();ctx.arc(p.x,p.y,2,0,Math.PI*2);ctx.fill();}
}
/* ---------- tick: office life; statuses come from real data ---------- */""")
rep("    chatter(s);\n  }", "    chatter(s);\n    if(Math.random()<.012&&!s.path.length){const k=s.status==='idle'?s.idle:s.status;const pool=REACT[k];if(pool)react(s,pick(pool));}\n  }")

open(dst, 'w', encoding='utf-8').write(h)
print('ok', len(h))

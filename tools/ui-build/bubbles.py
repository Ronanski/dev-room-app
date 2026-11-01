"""Bubbles never overlap: a per-frame layout pass lifts colliding bubbles (with a leader line) and a talk budget limits chatter."""
import sys
p = sys.argv[1]
h = open(p, encoding='utf-8').read()

def rep(old, new, count=1):
    global h
    n = h.count(old)
    assert n == count, f"expected {count} of {old[:90]!r}, found {n}"
    h = h.replace(old, new)

rep(".bub:empty{display:none}", ".bub:empty{display:none}\n.bub{position:relative}\n.ent.lifted .bub::after{content:\"\";position:absolute;left:50%;top:100%;width:1.5px;height:var(--lift,0);background:rgba(43,33,27,.55);transform:translateX(-50%)}")

# layout pass after positioning every label
rep("function updateOverlay(t){", r"""function layoutBubbles(){
  // Most important first: alarms, then the selected character, then whoever is nearest the camera.
  const items=[];
  for(const e of PEOPLE()){
    const n=nodes.get(e);if(!n)continue;
    if(n.style.display==='none'||!n._bt){if(n._lift){n._lift=0;n._b.style.translate='';n.classList.remove('lifted');}continue;}
    const key=n._bt+'|'+bsI+'|'+(n._e&&!n._e.hidden?1:0);if(n._bkey!==key){n._bkey=key;n._bw=n._b.offsetWidth;n._bh=n._b.offsetHeight;n._below=n.offsetHeight-(n._b.offsetTop+n._b.offsetHeight);}
    const x=parseFloat(n.style.left),y=parseFloat(n.style.top)-n._below;
    items.push({n,x,y,w:n._bw,h:n._bh,pri:(e.status==='needs'||e.status==='failed')?0:(e.id===selected?1:e.npc?3:2)});
  }
  items.sort((a,b)=>a.pri-b.pri||b.y-a.y);
  // name tags stay where they are, so bubbles must not cover them either
  const placed=[];
  for(const n of nodes.values()){if(n.style.display==='none'||!n._t.textContent)continue;const tk=n._t.textContent+'|'+bsI;if(n._tkey!==tk){n._tkey=tk;n._tw=n._t.offsetWidth;n._tht=n._t.offsetHeight;n._tb=n.offsetHeight-(n._t.offsetTop+n._t.offsetHeight);}
    const x=parseFloat(n.style.left),b=parseFloat(n.style.top)-n._tb;placed.push({l:x-n._tw/2,r:x+n._tw/2,t:b-n._tht,b});}
  for(const it of items){
    let lift=0;
    for(let k=0;k<14;k++){
      const t=it.y-it.h-lift,b=it.y-lift,l=it.x-it.w/2,r=it.x+it.w/2;
      const hit=placed.find(q=>l<q.r+4&&r>q.l-4&&t<q.b+3&&b>q.t-3);
      if(!hit)break;lift=it.y-hit.t+3;
    }
    placed.push({l:it.x-it.w/2,r:it.x+it.w/2,t:it.y-it.h-lift,b:it.y-lift});
    if(Math.abs((it.n._lift||0)-lift)>.5){it.n._lift=lift;it.n._b.style.translate=lift?`0 ${-lift}px`:'';it.n.classList.toggle('lifted',lift>0);it.n.style.setProperty('--lift',lift+'px');}
  }
}
function updateOverlay(t){""")
rep("  if(FX.effects)drawFx(t);", "  if(FX.effects)drawFx(t);\n  queueMicrotask?0:0;")
rep("  updateOverlay(t);\n  requestAnimationFrame(frame);", "  updateOverlay(t);layoutBubbles();\n  requestAnimationFrame(frame);")
rep("  if(FX.effects)drawFx(t);\n  queueMicrotask?0:0;", "  if(FX.effects)drawFx(t);")

# talk budget: fewer people speak at once when zoomed out; alarms and the selected character always may
rep("  if(s.idle==='rounds')return;\n  talk(s,lineFor(s));",
    "  if(s.idle==='rounds')return;\n  const live=PEOPLE().filter(o=>(o.talk&&now()<o.talk.until)||(o.npc&&now()<o.bUntil)).length,budget=cam.z<1.3?3:cam.z<2?5:8;\n  if(live>=budget&&s.status!=='needs'&&s.id!==selected){s.talkNext=now()+rand(3,8)*1000;return;}\n  talk(s,lineFor(s));")
# NPC chatter also respects the budget
rep("function say(e,text,ms){e.bubble=text;", "function say(e,text,ms){\n  if(e.npc&&e!==boss&&typeof cam!=='undefined'&&text!=='Zzz'){const live=PEOPLE().filter(o=>(o.talk&&now()<o.talk.until)||(o.npc&&now()<o.bUntil)).length;if(live>=(cam.z<1.3?3:cam.z<2?5:8)+1)return;}\n  e.bubble=text;")
open(p, 'w', encoding='utf-8').write(h)
print('ok')

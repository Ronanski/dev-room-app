"""Brighter floors, pinch zoom, more NPCs with routines, and characters that act out the real tool in use."""
import sys
p = sys.argv[1]
h = open(p, encoding='utf-8').read()

def rep(old, new, count=1):
    global h
    n = h.count(old)
    assert n == count, f"expected {count} of {old[:90]!r}, found {n}"
    h = h.replace(old, new)

# ---------- brighter floors, softer night ----------
for a, b in [("room(0,0,17,11,'#5d646c','#596068','tile')", "room(0,0,17,11,'#9aa1a8','#949ba2','tile')"),
             ("room(0,11,42,2,'#4b5157','#474d53')", "room(0,11,42,2,'#868c92','#81878d')"),
             ("room(28,13,6,10,'#4d5560','#48505a','tile')", "room(28,13,6,10,'#7d8590','#78808a','tile')"),
             ("room(35,0,7,11,'#6e6a72','#69656d','plank')", "room(35,0,7,11,'#a39dab','#9e98a6','plank')"),
             ("room(17,0,5,11,'#8a7462','#85705e','plank')", "room(17,0,5,11,'#b0997f','#ab947a','plank')"),
             ("room(22,0,6,11,'#7a5b46','#755743','plank')", "room(22,0,6,11,'#a8835f','#a37e5a','plank')"),
             ("room(28,0,7,11,'#9a7b5c','#957657','plank')", "room(28,0,7,11,'#b9966f','#b4916a','plank')"),
             ("room(16,13,6,10,'#8f7a66','#8a7562','plank')", "room(16,13,6,10,'#b29a82','#ad957d','plank')"),
             ("room(22,13,6,10,'#8a6f58','#856a54','plank')", "room(22,13,6,10,'#ad8d70','#a8886b','plank')"),
             ("const dark=x>=20||x<5?.32:x>=18.5?.18:x<6.5?.18:0;", "const dark=x>=20||x<5?.16:x>=18.5?.08:x<6.5?.08:0;")]:
    rep(a, b)

# ---------- pinch to zoom (two fingers), drag with one ----------
rep("cv.addEventListener('pointerdown',e=>{drag={sx:e.clientX,sy:e.clientY,cx:cam.tx,cy:cam.ty,moved:false};cv.setPointerCapture(e.pointerId);});",
    r"""const ptrs=new Map();let pinch=null;
const pinchState=()=>{const [a,b]=[...ptrs.values()];return{d:Math.hypot(a.x-b.x,a.y-b.y),mx:(a.x+b.x)/2,my:(a.y+b.y)/2};};
cv.addEventListener('pointerdown',e=>{ptrs.set(e.pointerId,{x:e.clientX,y:e.clientY});cv.setPointerCapture(e.pointerId);
  if(ptrs.size===2){const s=pinchState();pinch={d0:s.d,z0:cam.z};drag=null;cam.follow=null;return;}
  drag={sx:e.clientX,sy:e.clientY,cx:cam.tx,cy:cam.ty,moved:false};});
cv.addEventListener('pointermove',e=>{if(!ptrs.has(e.pointerId))return;ptrs.set(e.pointerId,{x:e.clientX,y:e.clientY});
  if(pinch&&ptrs.size===2){const s=pinchState();const q=toScreen({clientX:s.mx,clientY:s.my});setZoom(pinch.z0*s.d/Math.max(1,pinch.d0),q.x,q.y);cam.x=cam.tx;cam.y=cam.ty;}});
const endPtr=e=>{ptrs.delete(e.pointerId);if(ptrs.size<2&&pinch){pinch=null;drag=null;}};
cv.addEventListener('pointercancel',e=>{endPtr(e);drag=null;cv.classList.remove('drag');});""")
rep("cv.addEventListener('pointerup',e=>{\n  cv.classList.remove('drag');const d=drag;drag=null;if(!d||d.moved)return;",
    "cv.addEventListener('pointerup',e=>{\n  const wasPinch=!!pinch;endPtr(e);\n  cv.classList.remove('drag');const d=drag;drag=null;if(wasPinch||!d||d.moved)return;")

# ---------- characters act out the tool in use ----------
rep("const PEOPLE=()=>[...sessions,guard,hr,boss];", r"""// What the session is doing right now, read from the hook's tool label ("Nagko-code.", "Nagbabasa." ...).
function workMode(s){
  if(s.status!=='working')return'';
  const a=(s.activity&&s.activity[0]||'')+' '+(s.detail||'');
  if(/deleg|subagent/i.test(a))return'delegate';
  if(/research|web/i.test(a))return'web';
  if(/patakbo|test|build|run/i.test(a))return'run';
  if(/basa|hanap|read|search|review/i.test(a))return'read';
  if(/plano|todo/i.test(a))return'plan';
  return'code';
}
const NPCS=[];const couriers=[];
const PEOPLE=()=>[...sessions,guard,hr,boss,...NPCS,...couriers];""")
rep("    for(let i=0;i<3;i++)box(g,u+.15+i*.95,v+.06,.85,.12,11,'#15191d',8,{front:sc});",
    r"""    for(let i=0;i<3;i++)box(g,u+.15+i*.95,v+.06,.85,.12,11,'#15191d',8,{front:sc});
    const mode=s&&!s.path.length?workMode(s):'',vf=v+.18;
    const F=(u1,u2,z1,z2,c)=>poly(g,[iso(u1,vf,z1),iso(u2,vf,z1),iso(u2,vf,z2),iso(u1,vf,z2)],c);
    if(s&&s.flash&&now()<s.flash.until)for(let i=0;i<3;i++)F(u+.2+i*.95,u+.95+i*.95,9,23,s.flash.c);
    else if(mode==='code')for(let i=0;i<3;i++)for(let r=0;r<4;r++){const k=(t*2+r*.3+i)%1;F(u+.25+i*.95,u+.3+i*.95+.5*((r*7+i*3+Math.floor(t*3))%5)/5,22-r*3.2,21.2-r*3.2,['#9fd8e6','#e2c08d','#b6e3a8'][r%3]);}
    else if(mode==='read'){const sc2=(t*.6)%1;F(u+1.2,u+1.95,10,22,'#eef1f4');for(let r=0;r<4;r++)F(u+1.3,u+1.85,21-((r*3+sc2*3)%12),20.4-((r*3+sc2*3)%12),'#8a9097');}
    else if(mode==='run'){const k=(t*.25+s.seed)%1;F(u+.25,u+2.7,14,16.5,'#15191d');F(u+.25,u+.25+2.45*k,14,16.5,'#6fbf73');}
    else if(mode==='web'){const c=iso(u+1.55,vf,16);g.fillStyle='#4f8fd0';g.beginPath();g.ellipse(c.x,c.y,4.5,4.5,0,0,Math.PI*2);g.fill();g.strokeStyle='#cfe6ff';g.lineWidth=.6;g.beginPath();g.ellipse(c.x,c.y,2,4.5,0,0,Math.PI*2);g.stroke();}
    else if(mode==='plan'){for(let r=0;r<3;r++){F(u+1.25,u+1.4,20-r*4,18.8-r*4,r<Math.floor(t*.7)%4?'#6fbf73':'#8a9097');F(u+1.5,u+1.9,20-r*4,19.4-r*4,'#cfd6dc');}}""")
# typing only while writing code; delegating sends a courier (the subagent) to the library and back
rep("    chatter(s);\n    if(Math.random()<.012", r"""    {const md=workMode(s);if(s.pose==='sit'&&s.status==='working'){s.typing=md==='code'||md==='plan'||(md==='read'&&Math.floor(now()/1400+s.seed)%3===0);}
     if(md==='delegate'&&!s.courier&&!s.path.length){spawnCourier(s);}
     if(s.lastMode==='run'&&md!=='run'){const ok=/pumasa|pass|naayos|ok/i.test((s.recent||'')+(s.detail||''));const bad=s.status==='failed'||/fail|trip|error/i.test((s.recent||'')+(s.detail||''));if(ok||bad){s.flash={c:bad?'#d0473a':'#6fbf73',until:now()+5000};react(s,bad?'💥':'✅');}}
     s.lastMode=md;}
    chatter(s);
    if(Math.random()<.012""")
rep("function bossToDesk(){", r"""function spawnCourier(s){
  const seat=seatOf(s);const c={id:-100-couriers.length,npc:true,name:'Subagent',look:{hs:'bob',bottom:'pants'},pal:{hair:'#2b2b33',skin:'#e6bb94',shirt:'#9fb6c4',pants:'#3a3f45'},x:seat.x+T*.6,y:seat.y,path:[],pose:'stand',seed:Math.random()*4,bubble:'',bUntil:0,small:true};
  couriers.push(c);s.courier=c;say(c,`Ako na ang bahala, ${s.short}.`);
  const shelf={x:rand(29,34)*T,y:rand(1.6,3.2)*T};
  goTo(c,shelf,()=>{c.pose='stand';say(c,'Hinahanap ko ang kailangan mo.');
    setTimeout(()=>goTo(c,{x:seat.x+T*.9,y:seat.y},()=>{say(c,`Heto na ang report, ${s.short}.`);react(s,'📋');
      setTimeout(()=>{couriers.splice(couriers.indexOf(c),1);const n=nodes.get(c);if(n){n.remove();nodes.delete(c);}s.courier=null;},2500);}),8000/Math.max(1,SPEED()/3));});
}
function bossToDesk(){""")
rep("const hourNow=", "const hourNowX=0;\nconst hourNow=")

# ---------- NPC routines: janitor, barista, maintenance tech; guard patrols; safety rounds ----------
rep("function guardThink(){", r"""// Each NPC loops through a realistic routine: go somewhere, do the task (with a prop and a line), move on.
function mkNpc(o){const n=Object.assign({npc:true,path:[],pose:'stand',seed:Math.random()*4,bubble:'',bUntil:0,next:0,step:0},o);n.x=o.home.x;n.y=o.home.y;NPCS.push(n);return n;}
const janitor=mkNpc({id:-4,name:'Janitor',look:{hs:'bun',bottom:'pants'},pal:{hair:'#2b1d14',skin:'#b88458',shirt:'#5f7c78',pants:'#2e3338'},home:{x:6.4*T,y:15.4*T},prop:'mop',
  plan:[{at:()=>({x:rand(2,40)*T,y:rand(11.4,12.6)*T}),pose:'stand',act:'mop',say:['Basa ang sahig, ingat po.','Linis muna dito.'],dur:[40,70]},
        {at:()=>({x:7*T,y:20*T}),act:'mop',say:['Nililinis ko ang CR.'],dur:[40,60]},
        {at:()=>({x:rand(2,14)*T,y:rand(4.6,9.8)*T}),act:'trash',say:['Kukunin ko lang ang basura.','Pasensya na sa abala.'],dur:[15,25]},
        {at:()=>({x:rand(10.5,14.5)*T,y:rand(17,21)*T}),act:'wipe',say:['Pupunasan ko ang mesa.'],dur:[20,35]},
        {at:()=>({x:rand(35,41)*T,y:rand(14,21)*T}),act:'mop',say:['Ang daming pawis sa court.'],dur:[30,50]}]});
const barista=mkNpc({id:-5,name:'Pantry',look:{hs:'ponytail',bottom:'skirt'},pal:{hair:'#3a2a1a',skin:'#e0b08c',shirt:'#efe6d8',pants:'#5e3f2b'},home:{x:12.6*T,y:14.4*T},prop:'apron',
  plan:[{at:()=>({x:12.6*T,y:14.4*T}),act:'brew',say:['Bagong timpla ang kape.','Kape po? Mainit pa.'],dur:[40,80]},
        {at:()=>({x:rand(11,14)*T,y:rand(17.5,21.5)*T}),act:'tray',say:['Heto na po ang order.','Bagong luto ang ulam.'],dur:[15,30]},
        {at:()=>({x:rand(11,14)*T,y:rand(17.5,21.5)*T}),act:'wipe',say:['Lilinisin ko lang ang mesa.'],dur:[15,25]},
        {at:()=>({x:14.6*T,y:14.4*T}),act:'stock',say:['Kulang na ang tubig sa ref.'],dur:[20,30]}]});
const tech=mkNpc({id:-6,name:'Maintenance',look:{hs:'short'},pal:{hair:'#1e1712',skin:'#a97650',shirt:'#c26a2b',pants:'#2e3338'},home:{x:40.4*T,y:12.2*T},prop:'tool',
  plan:[{at:()=>{const d=pick(DESKS);return{x:(d.x+.4)*T,y:(d.y+1.3)*T};},act:'fix',say:['PM check lang po sa console.','Hihigpitan ko lang ang cable.'],dur:[40,70]},
        {at:()=>({x:rand(3,13)*T,y:1.9*T}),act:'fix',say:['Calibrate ko ang video wall.','May patay na pixel dito.'],dur:[40,60]},
        {at:()=>({x:29.8*T,y:16.2*T}),act:'fix',say:['Lalagyan ko ng langis ang treadmill.'],dur:[30,50]},
        {at:()=>({x:13.4*T,y:14.6*T}),act:'coffee',say:['Kape muna bago sa susunod na job order.'],dur:[30,50]}]});
function npcThink(n){
  if(n.path.length||now()<n.next)return;
  const st=n.plan[n.step++%n.plan.length];const to=st.at();
  if(n.step>1&&Math.random()<.25&&st.act!=='brew'){n.next=now()+rand(5,12)*1000;return;}
  goTo(n,to,()=>{n.pose='stand';n.task=st.act;say(n,pick(st.say));n.next=now()+rand(...st.dur)*1000/Math.max(1,SPEED()/3);});
  n.task='';
}
function guardThink(){""")
rep("function guardWake(){", r"""let guardPatrol=0;
function guardRound(){
  // every few minutes the guard walks the corridor with a flashlight, then signs the logbook
  if(guard.path.length||now()<guardPatrol)return false;
  guardPatrol=now()+rand(6,10)*MIN/Math.max(1,SPEED()/2);
  guard.state='patrol';guard.sleepy=false;guard.pose='walk';
  goTo(guard,{x:40*T,y:12*T},()=>{say(guard,'Clear ang east side.');setTimeout(()=>goTo(guard,{x:2.6*T,y:16*T},()=>{guard.pose='sitf';guard.state='ml';say(guard,'Pirma sa logbook: patrol, walang problema.');}),3000);});
  return true;
}
function guardWake(){""")
rep("  guard.sleepy=guard.state==='doze';\n  if(guard.path.length||now()<guard.next)return;",
    "  guard.sleepy=guard.state==='doze';\n  if(guard.path.length||guard.state==='patrol')return;\n  if(now()>guard.next&&Math.random()<.15&&guardRound())return;\n  if(now()<guard.next)return;")
rep("  hrThink();guardThink();bossThink();", "  hrThink();guardThink();bossThink();NPCS.forEach(npcThink);")
# safety officer carries a clipboard and does inspections when nobody is overworked
rep("  goTo(hr,wanderSpot(),()=>{hr.pose='stand';setTimeout(()=>goTo(hr,hr.home,()=>{hr.pose='stand';}),2500);});",
    "  const insp=pick([{x:30.5*T,y:17*T,l:'Inspection sa gym: ayos ang mga equipment.'},{x:38*T,y:19*T,l:'Walang madulas sa court? Ayos.'},{x:12.5*T,y:16*T,l:'Food safety check sa pantry.'},{x:8*T,y:6*T,l:'Housekeeping check sa control room.'}]);\n  goTo(hr,insp,()=>{hr.pose='stand';say(hr,insp.l);setTimeout(()=>goTo(hr,hr.home,()=>{hr.pose='stand';}),5000);});")
rep("const hr={id:-3,npc:true,name:'Safety',", "const hr={id:-3,npc:true,name:'Safety',prop:'clip',")

# ---------- props for NPCs ----------
rep("  if(e===hr){px(x-1,y0+7+bob,2,1,'#efe6d8');}", r"""  if(e===hr){px(x-1,y0+7+bob,2,1,'#efe6d8');}
  if(e.small){/* subagent courier: drawn with a folder */px(x+3,y0+8+bob,3,3,'#e2c08d');}
  if(e.prop==='clip'){px(x-7,y0+8+bob,3,4,'#c9a27e');px(x-7,y0+8+bob,3,1,'#5b6168');}
  if(e.prop==='apron'){px(x-3,y0+8+bob,6,4,'#5e3f2b');}
  if(e.prop==='tool'){px(x+4,y0+11+bob,4,2,'#c8382f');px(x+5,y0+10+bob,2,1,'#2a2f35');}
  if(e.prop==='mop'){const w=e.task==='mop'&&!moving?Math.floor(t*2+e.seed)%2:0;px(x+5+w,y0+2,1,12,'#9a7b5c');px(x+3+w,y0+13,4,2,'#d9d2c0');if(!moving)px(x+8,y0+11,3,3,'#3a6ea8');}
  if(e.task==='tray'&&!moving){px(x-6,y0+7,6,1,'#9aa3ab');px(x-5,y0+5,2,2,'#efe6d8');}
  if(e.task==='brew'&&!moving){const k=(t*.8)%1;px(x+4,y0+8,3,3,'#efe6d8');ctx.fillStyle=`rgba(239,230,216,${.6*(1-k)})`;}
  if(e.task==='trash'&&!moving){px(x-7,y0+9,3,4,'#2b3036');}""")
rep("function sprite(e,t){\n  const g=sg,p=e.pal,x=10,y=22,L=e.look;", "function sprite(e,t){\n  const g=sg,p=e.pal,x=10,y=22,L=e.look&&e.look.hs!=='short'?e.look:(e.look&&e.look.hs==='short'?null:e.look);")
# small courier drawn smaller
rep("  const x=Math.round(p.x-10*CS),y=Math.round(p.y-22*CS);const flip=",
    "  const CSX=e.small?1.4:CS;const x=Math.round(p.x-10*CSX),y=Math.round(p.y-22*CSX);const flip=")
rep("  for(const[dx,dy] of [[-1,0],[1,0],[0,-1],[0,1]])ctx.drawImage(sil,x+dx*CS,y+dy*CS,20*CS,24*CS);\n  ctx.drawImage(spr,x,y,20*CS,24*CS);\n  if(flip)ctx.restore();",
    "  for(const[dx,dy] of [[-1,0],[1,0],[0,-1],[0,1]])ctx.drawImage(sil,x+dx*CSX,y+dy*CSX,20*CSX,24*CSX);\n  ctx.drawImage(spr,x,y,20*CSX,24*CSX);\n  if(flip)ctx.restore();")
# icons for the new NPCs
rep("  if(e===boss)return{meet:'🗣️',watch:'👀',break:'☕'}[boss.mode]||'💼';",
    "  if(e===boss)return{meet:'🗣️',watch:'👀',break:'☕'}[boss.mode]||'💼';\n  if(e===janitor)return{mop:'🧹',trash:'🗑️',wipe:'🧽'}[e.task]||'🚶';\n  if(e===barista)return{brew:'☕',tray:'🍽️',wipe:'🧽',stock:'📦'}[e.task]||'🚶';\n  if(e===tech)return{fix:'🔧',coffee:'☕'}[e.task]||'🚶';\n  if(e.small)return'📁';\n  if(e===hr)return'📋';\n  if(e===guard&&guard.state==='patrol')return'🔦';")
rep("  if(e===guard)return guard.state==='doze'?'💤':'📱';", "  if(e===guard&&guard.state!=='patrol')return guard.state==='doze'?'💤':'📱';")

# ---------- richer sample events, so the synced work modes show up ----------
rep("""const SCRIPT=[""", """const SCRIPT=[
 ['A',{status_bucket:SW_,short:'Nagde-delegate.',post_turn_summary:{status_detail:'Nag-utos sa subagent: hanapin ang lahat ng tawag sa tripLogic.'}}],
 ['D',{status_bucket:SW_,short:'Nagre-research.',post_turn_summary:{status_detail:'Nagbabasa sa web tungkol sa ISA-18.2.'}}],
 ['C',{status_bucket:SW_,short:'Nagpapatakbo.',post_turn_summary:{status_detail:'Pinapatakbo ang npm test.'}}],
 ['L',{status_bucket:SW_,short:'Nagbabasa.',post_turn_summary:{status_detail:'Binabasa ang PR #91.'}}],
 ['C',{status_bucket:SI_,post_turn_summary:{recent_action:'Pumasa ang 58 tests.'}}],""")

open(p, 'w', encoding='utf-8').write(h)
print('ok')

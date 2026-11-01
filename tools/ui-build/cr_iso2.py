"""Turn the isometric 0.4 preview into the Control Room theme (isometric)."""
import sys
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

rep('<title>Dev Room 0.4 Preview</title>', '<title>Dev Room Control Room Iso</title>')
rep('<p class="sub">Sample na office · ', '<p class="sub">Unit 1 control room · sample na shift · ')
rep('<h1>Dev Room</h1>', '<h1>Dev Room · Unit 1</h1>')
rep("""el.innerHTML=`<span class="mocktag">Preview 0.4</span><span class="cm">Sample na team ito na may kunwaring events kada 20 segundo. I-off ang isang feature para makita ang pagkakaiba.</span>""",
    """el.innerHTML=`<span class="mocktag">Control Room</span><span class="cm">Isometric na mockup. Sample na shift na may kunwaring events kada 20 segundo. Nasa Alarms tab ang mga naghihintay sa iyo, at may ACK ang bawat isa.</span>""")

# ---------- plant wording ----------
rep("const ST={working:{label:'Nagtatrabaho',c:'--work'},needs:{label:'Kailangan ka',c:'--need'},idle:{label:'Naka-break',c:'--idle'},done:{label:'Tapos na',c:'--done'},failed:{label:'May error',c:'--fail'}};",
    "const ST={working:{label:'Running',c:'--work'},needs:{label:'Alarm',c:'--need'},idle:{label:'Standby',c:'--idle'},done:{label:'Pa-review',c:'--done'},failed:{label:'Trip',c:'--fail'}};")

# ---------- layout: control room, rest room, corridor, gate, locker, rec room, mess room ----------
rep("const T=16,W=480,H=288,CORR=144;", "const T=16,W=672,H=368,CORR=192;")
rep("const SW=1120,SH=630,RES=2,TW=22,TH=11,OX=18*22+20,OY=86,WALL=64,HS=1.4;", "const SW=1540,SH=820,RES=2,TW=22,TH=11,OX=23*22+24,OY=86,WALL=64,HS=1.4;")
rep("const BGRES=3;const bg=document.createElement('canvas');bg.width=SW*BGRES;bg.height=SH*BGRES;", "const BGRES=2;const bg=document.createElement('canvas');bg.width=SW*BGRES;bg.height=SH*BGRES;")
between("const DESKS=", "const taken={};", r"""// Control room: two rows of three consoles facing the video wall.
const DESKS=[[1.4,2.6],[5.8,2.6],[10.2,2.6],[1.4,6.2],[5.8,6.2],[10.2,6.2],[14.0,2.6],[14.0,6.2]].map(([x,y],i)=>({i,x,y,w:3,seat:{x:(x+1.5)*T,y:(y+1.55)*T},owner:null}));
const LEAD_DESK={x:18.2,y:4.4,w:3,seat:{x:19.7*T,y:5.95*T},owner:null};
const BOSS_POS={x:25*T,y:3.95*T};
const SPOTS={
 queue:[{x:23.4*T,y:11.7*T},{x:25.0*T,y:11.7*T},{x:26.6*T,y:11.7*T}],
 couch:[{x:36.4*T,y:3.3*T},{x:36.4*T,y:6.3*T},{x:39.6*T,y:3.3*T},{x:39.6*T,y:6.3*T}],
 eat:[{x:11.4*T,y:18.6*T},{x:13.1*T,y:18.6*T},{x:11.4*T,y:20.8*T},{x:13.1*T,y:20.8*T}],
 chismis:[{x:16.6*T,y:12.2*T},{x:17.9*T,y:12.2*T},{x:17.2*T,y:12.6*T}],
 coffee:[{x:10.2*T,y:14.3*T},{x:11.6*T,y:14.3*T}],
 cr:[{x:6.2*T,y:21.5*T},{x:7.8*T,y:21.5*T}],
 tv:[{x:17.6*T,y:17.3*T},{x:19.0*T,y:17.3*T},{x:20.4*T,y:17.3*T}],
 billiards:[{x:22.9*T,y:15.9*T},{x:26.4*T,y:15.9*T}],
 darts:[{x:26.0*T,y:20.7*T}],
 chess:[{x:23.2*T,y:21.0*T},{x:24.9*T,y:20.4*T}],
 read:[{x:30.2*T,y:7.0*T},{x:32.4*T,y:7.3*T},{x:30.9*T,y:9.3*T}],
 gym:[{x:29.4*T,y:15.3*T},{x:31.0*T,y:15.3*T},{x:32.5*T,y:19.2*T}],
 basketball:[{x:37.0*T,y:17.2*T},{x:39.6*T,y:18.6*T},{x:36.2*T,y:20.0*T}]};
const BOSS_SEAT={x:25*T,y:3.95*T},BOSS_MEET={x:25*T,y:7.6*T};
""")
rep("function roomOf(p){if(p.y>H)return'out';if(p.y<8*T)return p.x<19*T?'work':p.x<24*T?'lead':'boss';if(p.y<10*T)return'corr';const tx=p.x/T;return tx",
    "function roomOf(p){if(p.y>H)return'out';const tx=p.x/T;if(p.y<11*T)return tx<17?'work':tx<22?'lead':tx<28?'boss':tx<35?'lib':'sleep';if(p.y<13*T)return'corr';return tx<5?'lobby':tx<9?'cr':tx<16?'pantry':tx<22?'lounge':tx<28?'rec':tx<34?'gym':'court';return tx")
rep("function aisle(p){const g=[5,9,13,17].map(v=>v*T);", "function aisle(p){const g=[.8,5.1,9.5,13.6].map(v=>v*T);")
rep("function wanderSpot(){return{x:rand(2,23)*T,y:rand(8.6,9.6)*T};}", "function wanderSpot(){return{x:rand(2,40)*T,y:rand(11.4,12.6)*T};}")

# ---------- people ----------
rep("x:12.6*T,y:14.2*T,path:[],pose:'sitf',seed:1", "x:2.6*T,y:16.0*T,path:[],pose:'sitf',seed:1")
rep("const hr={id:-3,npc:true,name:'HR',", "const hr={id:-3,npc:true,name:'Safety',")
h = h.replace("x:19.5*T,y:13.9*T,path:[]", "x:40.6*T,y:9.4*T,path:[]").replace("home:{x:19.5*T,y:13.9*T}", "home:{x:40.6*T,y:9.4*T}")
rep("say(hr,`Magpahinga ka muna. ${m} ka nang tutok.`,3500);log('HR'", "say(hr,`Fatigue check: ${m} ka nang tuloy-tuloy. Pahinga muna.`,3500);log('Safety'")
rep("else{place(s,{x:14.5*T,y:H+12});", "else{place(s,{x:2.5*T,y:H+12});")
rep("iso(22,7);c", "iso(14,9);c")
rep("resizeMap();{const p=iso(21,7);", "resizeMap();{const p=iso(14,9);")

# ---------- world: floors, walls, video wall, consoles, rooms ----------
between("/* static floor + back walls */", "/* characters: pixel sprite with a dark outline */", r"""/* static floor + back walls */
const BGRES=2;const bg=document.createElement('canvas');bg.width=SW*BGRES;bg.height=SH*BGRES;
(function drawBG(){
  const g=bg.getContext('2d');g.scale(BGRES,BGRES);
  const room=(u0,v0,w,d,a,b,style)=>{for(let u=u0;u<u0+w;u++)for(let v=v0;v<v0+d;v++){
    const c=style==='tile'?((u+v)%2?a:b):(v%2?a:b);flat(g,u,v,1,1,c);
    if(style==='plank'){g.strokeStyle='rgba(20,22,26,.18)';g.lineWidth=.6;const m1=iso(u,v+.5),m2=iso(u+1,v+.5);g.beginPath();g.moveTo(m1.x,m1.y);g.lineTo(m2.x,m2.y);g.stroke();}}};
  room(0,0,17,11,'#5d646c','#596068','tile');    // control room
  room(17,0,5,11,'#8a7462','#85705e','plank');   // team supervisor office
  room(22,0,6,11,'#7a5b46','#755743','plank');   // your office
  room(28,0,7,11,'#9a7b5c','#957657','plank');   // library
  room(35,0,7,11,'#6e6a72','#69656d','plank');   // sleeping quarters
  room(0,11,42,2,'#4b5157','#474d53');            // corridor
  room(0,13,5,10,'#8c8f93','#83868a','tile');     // gate
  room(5,13,4,10,'#cfd3d6','#c3c8cc','tile');     // CR and lockers
  room(9,13,7,10,'#c9bca6','#bcae97','tile');     // pantry
  room(16,13,6,10,'#8f7a66','#8a7562','plank');   // lounge
  room(22,13,6,10,'#8a6f58','#856a54','plank');   // rec room
  room(28,13,6,10,'#4d5560','#48505a','tile');    // gym (rubber floor)
  room(34,13,8,10,'#c48a52','#bd844d','plank');   // basketball half court
  // court markings
  g.strokeStyle='rgba(255,255,255,.75)';g.lineWidth=1.1;
  const L=(pts)=>{g.beginPath();pts.forEach(([u,v],i)=>{const p=iso(u,v);i?g.lineTo(p.x,p.y):g.moveTo(p.x,p.y);});g.stroke();};
  L([[34.6,13.6],[41.4,13.6],[41.4,22.4],[34.6,22.4],[34.6,13.6]]);
  L([[36.4,13.6],[36.4,17.6],[39.6,17.6],[39.6,13.6]]);
  const arc=[];for(let k=0;k<=24;k++){const a=Math.PI*k/24;arc.push([38+3.6*Math.cos(a),13.6+3.9*Math.sin(a)]);}L(arc);
  const key=[];for(let k=0;k<=16;k++){const a=Math.PI*k/16;key.push([38+1.6*Math.cos(a),17.6+1.6*Math.sin(a)]);}L(key);
  // rugs, carpet, safety line
  flat(g,22.6,5.2,4.8,2.6,'#5e4a3c','rgba(20,24,28,.4)');flat(g,22.8,5.4,4.4,2.2,'#6e5847');
  flat(g,29,5.6,5,3.6,'#6b5f4e','rgba(20,24,28,.35)');
  flat(g,16.4,15.6,5.2,3.2,'#6f6a52','rgba(20,24,28,.4)');flat(g,16.6,15.8,4.8,2.8,'#7d7a5e');
  flat(g,0.3,1.6,16.4,.12,'#d6b23a');
  flat(g,1,15,3,.6,'#2e3338');
  // back walls
  const wallFace=(p1,p2,p3,p4,col)=>poly(g,[p1,p2,p3,p4],col,OUT);
  wallFace(iso(0,0),iso(42,0),iso(42,0,WALL),iso(0,0,WALL),'#7d848b');
  wallFace(iso(0,0),iso(0,23),iso(0,23,WALL),iso(0,0,WALL),'#6c737a');
  g.strokeStyle='rgba(30,34,38,.28)';g.lineWidth=.6;
  for(let u=2;u<42;u+=2){const a=iso(u,0,0),b=iso(u,0,WALL);g.beginPath();g.moveTo(a.x,a.y);g.lineTo(b.x,b.y);g.stroke();}
  for(let v=2;v<23;v+=2){const a=iso(0,v,0),b=iso(0,v,WALL);g.beginPath();g.moveTo(a.x,a.y);g.lineTo(b.x,b.y);g.stroke();}
  wallFace(iso(0,0,WALL),iso(42,0,WALL),iso(42,-.25,WALL),iso(0,-.25,WALL),'#3a3f45');
  wallFace(iso(0,0,WALL),iso(0,23,WALL),iso(-.25,23,WALL),iso(-.25,0,WALL),'#3a3f45');
  wallFace(iso(1.5,0,10),iso(15.5,0,10),iso(15.5,0,58),iso(1.5,0,58),'#15191d');   // video wall frame
  const winV=(u1,u2,z1,z2,col)=>poly(g,[iso(u1,0,z1),iso(u2,0,z1),iso(u2,0,z2),iso(u1,0,z2)],col,OUT);
  winV(18,21,18,48,'#9fb6c4');winV(23,24.6,24,44,'#c9a27e');winV(25.4,27,24,44,'#7d8a6a');winV(36.5,40.5,18,48,'#9fb6c4');
  g.font='bold 9px monospace';g.fillStyle='rgba(15,18,22,.55)';g.textAlign='center';
  [['CONTROL ROOM',4,10.5],['TEAM SUPERVISOR',19.5,10.5],['OPISINA MO',25,10.5],['LIBRARY',31.5,10.5],['TULUGAN',38.5,10.5],
   ['GATE',2.5,22.4],['CR',7,22.4],['PANTRY',12.5,22.4],['LOUNGE',19,22.4],['REC ROOM',25,22.4],['GYM',31,22.4],['HALF COURT',38,22.6]].forEach(([t,u,v])=>{const p=iso(u,v);g.fillText(t,p.x,p.y);});
})();

/* furniture as depth-sorted objects */
const objs=[];
const add=(d,fn)=>objs.push({d,draw:fn});
const addBox=(u,v,w,d,h,col,z0,opt,depth)=>add(depth??(u+w/2+v+d/2),g=>box(g,u,v,w,d,h,col,z0,opt));
function wallSeg(u1,v1,u2,v2,h,glass){
  const th=.16,opt=glass?{front:'rgba(200,222,230,.35)',side:'rgba(170,200,210,.35)',top:'#9fb7c0'}:{top:'#596068'};
  if(v1===v2)for(let u=u1;u<u2-1e-6;u+=1){const w=Math.min(1,u2-u);addBox(u,v1-th/2,w,th,h,'#454b52',0,opt);}
  else for(let v=v1;v<v2-1e-6;v+=1){const d=Math.min(1,v2-v);addBox(u1-th/2,v,th,d,h,'#454b52',0,opt);}
}
// top band partitions (glass for the control room and supervisor), front walls with doors to the corridor
wallSeg(17,0,17,4,18,true);wallSeg(17,6,17,11,18,true);wallSeg(22,0,22,11,18,true);wallSeg(28,0,28,11,18);wallSeg(35,0,35,11,18);
wallSeg(0,11,1,11,10);wallSeg(4,11,8,11,10);wallSeg(11,11,13,11,10);wallSeg(17,11,18.5,11,10);wallSeg(21,11,23,11,10);wallSeg(27,11,29,11,10);wallSeg(31,11,32,11,10);wallSeg(34.5,11,36,11,10);wallSeg(39,11,42,11,10);
// bottom band
wallSeg(0,13,1.5,13,9);wallSeg(3.5,13,5.5,13,9);wallSeg(8,13,9.5,13,9);wallSeg(14,13,16.5,13,9);wallSeg(19.5,13,22.5,13,9);wallSeg(25.5,13,28.5,13,9);wallSeg(32,13,34.5,13,9);
for(const x of [5,9,16,22,28,34])wallSeg(x,13,x,23,10);
wallSeg(42,0,42,23,5);wallSeg(0,23,1.5,23,5);wallSeg(3.5,23,42,23,5);

function screenColor(s,t){
  if(!s||s.path.length||s.status==='idle'||s.status==='needs')return'#26313a';
  if(s.status==='working')return Math.floor(t*3+s.seed)%2?'#3f8aa5':'#367b94';
  if(s.status==='failed')return Math.floor(t*4)%2?'#d0473a':'#6a1f19';
  if(s.status==='done')return'#5f8a6c';
  return'#26313a';
}
function addConsole(dk,ownerFn){
  const u=dk.x,v=dk.y+.25,w=dk.w;
  add(u+w/2+v+.45,(g,t)=>{
    box(g,u,v,w,.8,8,'#3c4249',0,{top:'#4d545c'});
    const s=ownerFn(),sc=screenColor(s,t);
    for(let i=0;i<3;i++)box(g,u+.15+i*.95,v+.06,.85,.12,11,'#15191d',8,{front:sc});
    box(g,u+w/2-.45,v+.45,.9,.22,.6,'#262b31',8);
    if(s&&(s.status==='needs'||s.status==='failed')){const un=!s.acked;const on=!un||Math.floor(t*2)%2;box(g,u+w-.25,v+.1,.18,.18,3,'#2a2f35',8,{top:on?(s.status==='failed'||s.prio===1?'#ff4a3e':'#f2b23a'):'#3a2020',front:on?(s.status==='failed'?'#c8382f':'#d79a1c'):'#3a2020'});}
    if(s&&s.status==='failed'&&!s.path.length){for(let i=0;i<3;i++){const k=(t*.8+i/3)%1;const p=iso(u+w/2,v+.1,24+k*16);g.fillStyle=`rgba(120,118,115,${1-k})`;g.fillRect(p.x-2+i*2+Math.sin(k*6)*2,p.y,3,3);}}
  });
  const sx=dk.seat.x/T,sy=dk.seat.y/T;
  add(sx+sy-.25,g=>box(g,sx-.25,sy-.32,.5,.45,6,'#2b3036'));
}
DESKS.forEach(d=>addConsole(d,()=>d.owner));
addConsole(LEAD_DESK,()=>sessions.find(s=>s.lead));
addBox(17.4,1.0,.6,2.4,22,'#4a3f36');addBox(20.6,8.6,1,1,6,'#6b5c4e');
// your office: chair, desk (screen to the side so your face shows), bookshelf, couch
add(24.7+.3+3.7+.2,g=>{box(g,24.7,3.7,.6,.4,7,'#2a201a');box(g,24.7,3.7,.6,.12,18,'#2a201a',7);});
add(25+4.65,(g,t)=>{box(g,23.5,4.3,3,.7,9,'#6b4a35',0,{top:'#7d5840'});box(g,23.7,4.42,.6,.08,9,'#9a9086',9);box(g,26.2,4.5,.18,.18,4,'#efe6d8',9);box(g,24.6,4.55,.5,.3,1,'#d9cfbd',9);
  if(Math.floor(t*1.5)%2){const p=iso(26.3,4.6,15);g.fillStyle='rgba(239,230,216,.7)';g.fillRect(p.x,p.y,1,3);}});
add(27.1+1.2+.5,g=>{box(g,27.1,1.2,.7,2.6,30,'#4a3122');for(let i=0;i<3;i++)box(g,27.15,1.4+i*.8,.1,.6,5,['#7d8a6a','#9a6450','#c9a27e'][i],8+i*8);});
addBox(22.5,8.8,3,.8,5,'#7a6a5c',0,{top:'#8a7a6c'});
// library: shelves, reading table, armchairs
for(let i=0;i<5;i++)add(28.6+i*1.3+.6+.5,g=>{box(g,28.6+i*1.3,.3,1.1,.5,34,'#4a3122');for(let r=0;r<4;r++)for(let k=0;k<3;k++)box(g,28.68+i*1.3+k*.33,.32,.28,.1,6,['#9a6450','#5f7c78','#c9a27e','#7d8a6a'][(i+r+k)%4],4+r*8);});
addBox(30,6.0,2.6,1.2,7,'#6b4a35',0,{top:'#7d5840'});addBox(30.4,6.3,.6,.4,1,'#efe6d8',7);
addBox(33.4,8.4,.9,.9,6,'#7a4a3a');
// sleeping quarters: bunks, lockers, safety officer's desk
const bunk=(u,v)=>add(u+v+1.3,g=>{box(g,u,v,2.2,1,4,'#5b6570',0,{top:'#c9d0d8'});box(g,u,v,.3,1,9,'#46505a');box(g,u+.05,v+.1,.5,.8,1,'#eef1f4',4);});
bunk(35.6,2.6);bunk(35.6,5.6);bunk(38.8,2.6);bunk(38.8,5.6);
addBox(39.4,8.4,2.2,.7,9,'#5e5248',0,{top:'#6e6157'});addBox(40,8.45,.7,.1,9,'#15191d',9,{front:'#3f8aa5'});
// corridor: water cooler
add(17.2+11.3+.4,g=>{box(g,17.2,11.3,.4,.4,14,'#e6e0d4');box(g,17.25,11.35,.3,.3,7,'#bcd3d6',14);});
// gate
addBox(1.8,15.2,1.9,.6,9,'#3c4249',0,{top:'#4d545c'});addBox(2.3,15.25,.6,.1,8,'#15191d',9,{front:'#2f5c70'});
addBox(3.9,14.0,.15,1.2,8,'#9aa3ab');addBox(3.9,16.4,.15,1.2,8,'#9aa3ab');
// CR and lockers
for(let i=0;i<3;i++)addBox(5.3+i*.7,13.3,.6,.45,22,['#6f7f8a','#66757f','#6f7f8a'][i]);
addBox(5.6,21.8,1.2,.8,9,'#b9b0a2');addBox(7.3,21.8,1.2,.8,9,'#b9b0a2');addBox(5.4,17.0,.5,1.6,8,'#e8e4dc');
// pantry: counter, fridge, coffee machine, tables
add(9.4+13.6+.4,(g,t)=>{box(g,9.4,13.3,4.2,.7,10,'#4a3f36',0,{top:'#6b5a4c'});box(g,9.7,13.35,.6,.45,8,'#8a7f72',10);box(g,11.4,13.35,.9,.5,5,'#d9cfbd',10);
  if(Math.floor(t*1.5)%2){const p=iso(10,13.6,22);g.fillStyle='rgba(239,230,216,.7)';g.fillRect(p.x,p.y,1,3);}});
addBox(14.2,13.3,1.2,.8,26,'#d9dde0');
const table=(u,v)=>add(u+v+.9,g=>{box(g,u+.35,v+.35,.2,.2,7,'#3a2a20');box(g,u,v,.9,.9,2,'#7a5338',7,{top:'#8d6443'});box(g,u+.2,v+.2,.15,.15,3,'#efe6d8',9);box(g,u+.55,v+.5,.15,.15,3,'#efe6d8',9);});
table(11.8,18.8);table(11.8,21.0);
// lounge: TV wall, sofas, plants
let tvCh=0;const TVC=['#6f8fb0','#b08a5f','#7fa07a','#a06f8a','#c9b27a'];
add(17.2+13.2,(g,t)=>{box(g,17.2,13.15,3.6,.4,5,'#3a2a20');const on=sessions.some(s=>s.idle==='tv'&&!s.path.length);const c=on?(Math.floor(t*2)%2?TVC[tvCh%TVC.length]:shade(TVC[tvCh%TVC.length],.85)):'#25282c';box(g,17.4,13.2,3.2,.12,12,'#1d1d22',5,{front:c});});
addBox(16.8,18.3,4.4,.7,5,'#6b4a35',0,{top:'#7d5840'});addBox(16.8,18.3,4.4,.25,10,'#5a3e2c');
addBox(16.4,21.4,2.4,.8,5,'#6b4a35',0,{top:'#7d5840'});
// rec room: billiards, chess, darts
add(23.4+15.05+.65,g=>{box(g,23.5,14.5,.25,.25,6,'#3a2a20');box(g,25.5,14.5,.25,.25,6,'#3a2a20');box(g,23.5,15.5,.25,.25,6,'#3a2a20');box(g,25.5,15.5,.25,.25,6,'#3a2a20');
  box(g,23.4,14.4,2.4,1.3,2,'#5e3f2b',6,{top:'#4f7a52'});box(g,24,14.8,.12,.12,1,'#efe6d8',8);box(g,24.8,15.0,.12,.12,1,'#b03a2e',8);box(g,25.1,14.7,.12,.12,1,'#e2c08d',8);});
add(23.6+20.4+.9,g=>{box(g,23.95,20.75,.2,.2,6,'#3a2a20');box(g,23.6,20.4,.9,.9,1.5,'#5e3f2b',6,{top:'#d9cfbd'});for(let i=0;i<4;i++)box(g,23.7+(i%2)*.4,20.5+(i>>1)*.4,.12,.12,1.5,i%3?'#2a201a':'#efe6d8',7.5);});
add(27.84+20.2+.5,g=>{box(g,27.8,20.0,.06,.7,6,'#2a201a',5,{front:'#b03a2e'});box(g,27.79,20.25,.06,.2,2,'#efe6d8',7);});
// gym: treadmills, bench, dumbbell rack, mirror
const tread=(u,v)=>add(u+v+.9,g=>{box(g,u,v,.9,1.6,2,'#2b3036',0,{top:'#3c4249'});box(g,u,v,.9,.15,10,'#5b6168');box(g,u+.1,v,.7,.1,3,'#15191d',10,{front:'#3f8aa5'});});
tread(29,14.2);tread(30.6,14.2);
addBox(31.6,18.6,1.8,.5,4,'#2b3036',0,{top:'#7a4a3a'});addBox(32.1,18.4,.12,.9,9,'#9aa3ab');addBox(32.9,18.4,.12,.9,9,'#9aa3ab');
add(28.4+21.4+.4,g=>{box(g,28.4,21.4,2.2,.5,6,'#3c4249');for(let i=0;i<5;i++)box(g,28.5+i*.42,21.45,.3,.3,2,'#15191d',6);});
// basketball: hoop and backboard on the far side, bench
add(38+13.6,(g,t)=>{box(g,37.9,13.25,.2,.2,40,'#5b6168');box(g,37.2,13.5,1.6,.1,12,'#eef1f4',38,{front:'#eef1f4'});box(g,37.75,13.52,.5,.06,5,'#d0473a',40);
  const r=iso(38,14.2,44);g.strokeStyle='#d0473a';g.lineWidth=1.4;g.beginPath();g.ellipse(r.x,r.y,8,3.5,0,0,Math.PI*2);g.stroke();});
addBox(34.6,21.8,3,.6,5,'#6b4a35',0,{top:'#7d5840'});
// plants
const plant=(u,v)=>add(u+v+.5,g=>{box(g,u,v,.5,.5,6,'#7a5338');box(g,u+.05,v+.05,.4,.4,10,'#5f7a4f',6,{top:'#7a9466'});});
[[.4,.4],[16.2,.4],[21.3,.4],[34.3,.4],[41.3,9.9],[.4,13.4],[15.3,13.4],[21.3,22.2],[33.3,22.2]].forEach(([u,v])=>plant(u,v));
// video wall content
function wallText(u,z,txt,size,col){const p=iso(u,0,z);ctx.save();ctx.transform(1,TH/TW,0,1,p.x,p.y);ctx.fillStyle=col;ctx.font=`bold ${size}px monospace`;ctx.textAlign='center';ctx.fillText(txt,0,0);ctx.restore();}
let MWV=0;
add(-1e9,(g,t)=>{
  const P=(u1,u2,z1,z2,col)=>poly(g,[iso(u1,0,z1),iso(u2,0,z1),iso(u2,0,z2),iso(u1,0,z2)],col);
  P(2,6.4,13,55,'#1d2a33');P(6.8,11,13,55,'#1d2a33');P(11.4,15,13,55,'#1d2a33');
  sessions.slice(0,9).forEach((s,i)=>{const u=2.3+(i%3)*1.4,z=48-Math.floor(i/3)*12;const c=s.status==='working'?'#c9d2da':s.status==='failed'?(Math.floor(t*2)%2?'#ff4a3e':'#7a1d18'):s.status==='needs'?((s.acked||Math.floor(t*2)%2)?(s.prio===1?'#ff4a3e':'#f2b23a'):'#3a3020'):'#55606a';P(u,u+1.1,z-8,z,c);});
  const target=sessions.filter(s=>s.status==='working').length*(55+10*Math.sin(t*.3));MWV+=(target-MWV)*.02;
  wallText(8.9,36,String(Math.round(MWV)).padStart(3,'0')+' MW',12,'#9fd8e6');wallText(8.9,22,'GEN OUTPUT',6,'#6f8a96');
  if(!window.__tr)window.__tr=[];const tr=window.__tr;if(!tr.t||t-tr.t>1){tr.t=t;tr.push(MWV);if(tr.length>40)tr.shift();}
  g.strokeStyle='#9fd8e6';g.lineWidth=1;g.beginPath();tr.forEach((v,i)=>{const p=iso(11.7+i*(3/39),0,18+Math.min(34,v/600*34*2.2));i?g.lineTo(p.x,p.y):g.moveTo(p.x,p.y);});g.stroke();
  const un=sessions.some(s=>(s.status==='needs'||s.status==='failed')&&!s.acked);
  box(g,8.7,.05,.4,.3,2,'#2a2f35',60,{top:un&&Math.floor(t*3)%2?'#ff4a3e':'#5a2222',front:un&&Math.floor(t*3)%2?'#e0392f':'#4a1c1c'});
  if(un){const c=iso(8.9,.2,62),a=(t*4)%(Math.PI*2);g.fillStyle='rgba(255,74,62,.08)';g.beginPath();g.moveTo(c.x,c.y);g.arc(c.x,c.y,260,a,a+.5);g.fill();}
});

""")

# ---------- effects positions ----------
rep("u=4.15+1.9*(.5+.5*Math.sin(t*1.3*sp)),v=11.9+.85*(.5+.5*Math.sin(t*.9*sp+1));const p=iso(u,v,11.4);",
    "u=23.65+1.9*(.5+.5*Math.sin(t*1.3*sp)),v=14.6+.85*(.5+.5*Math.sin(t*.9*sp+1));const p=iso(u,v,11.4);")
rep("b=iso(6.8,16.35,11)", "b=iso(27.8,20.35,11)")

# ---------- alarms: ACK state, priority, and an Alarms tab ----------
rep("  const prev=s.status;s.status=st;s.since=s.at||now();",
    "  const prev=s.status;s.status=st;s.since=s.at||now();\n  if(st==='needs'||st==='failed'){s.acked=false;s.alarmAt=now();s.prio=/permission/i.test(s.ask||'')||st==='failed'?1:2;}")
rep('<button type="button" role="tab" data-ptab="chat" aria-selected="true">Chat</button>',
    '<button type="button" role="tab" data-ptab="alarms" aria-selected="true">Alarms</button>\n            <button type="button" role="tab" data-ptab="chat" aria-selected="false">Chat</button>')
rep("var ptab='chat',chatKey='';", "var ptab='alarms',chatKey='';")
rep("  if(ptab==='chat'){\n    const key=`c${s.id}", r"""  if(ptab==='alarms'){
    const al=sessions.filter(x=>x.status==='needs'||x.status==='failed').sort((a,b)=>(a.acked-b.acked)||((a.prio||2)-(b.prio||2))||(b.alarmAt||0)-(a.alarmAt||0));
    const key='a'+al.map(x=>x.id+':'+x.status+':'+(x.acked?1:0)).join('|');
    if(key!==chatKey){chatKey=key;body.innerHTML=al.length?`<ul class="alist">${al.map(x=>`<li class="alrow ${x.status==='failed'||x.prio===1?'p1':'p2'} ${x.acked?'':'unack'}" data-pick="${x.id}"><i></i><span class="m"><b>${esc(x.short)} · ${x.status==='failed'?'TRIP':'NEEDS INPUT'} · ${x.status==='failed'||x.prio===1?'HIGH':'MEDIUM'}</b><span>${esc(x.status==='failed'?(x.detail||'May pumalyang command.'):(x.ask||'May tanong sa iyo.'))}</span></span>${x.acked?'<span class="ackd">ACK</span>':`<button type="button" class="btn" data-ack="${x.id}">ACK</button>`}</li>`).join('')}</ul>`:'<p class="sub2">Walang alarm. Normal ang lahat ng operator.</p>';}
  }else if(ptab==='chat'){
    const key=`c${s.id}""")
rep("/* app build: no device preview, no in-app reply yet */", r""".alist{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:6px}
.alrow{display:grid;grid-template-columns:6px minmax(0,1fr) auto;gap:10px;align-items:center;padding:8px 10px;border:1px solid var(--line);border-radius:8px;background:var(--bg);cursor:pointer}
.alrow i{width:6px;align-self:stretch;border-radius:3px;background:var(--idle)}
.alrow.p1 i{background:var(--need)}.alrow.p2 i{background:#d99a1c}
.alrow.unack i{animation:blinkA 1s steps(2,start) infinite}
@keyframes blinkA{to{visibility:hidden}}
.alrow .m{min-width:0;font-size:13px}.alrow .m b{display:block;font-family:var(--f-mono);font-size:11px}
.ackd{font-family:var(--f-mono);font-size:11px;color:var(--muted)}
/* app build: no device preview, no in-app reply yet */""")
rep("document.addEventListener('click',e=>{", """document.addEventListener('click',e=>{
  const ak=e.target.closest('[data-ack]');if(ak){const s=sessions.find(x=>x.id===+ak.dataset.ack);if(s){s.acked=true;log('Ikaw',`ACK: alarm ni ${s.short}.`,s);chatKey='';render();}return;}
  const pk=e.target.closest('[data-pick]');if(pk&&!e.target.closest('[data-ack]')){selected=+pk.dataset.pick;const fs=sessions.find(x=>x.id===selected);if(fs){cam.follow=fs;}chatKey='';render();return;}""")
# beacon line on the counts strip
rep("""    <span class="count need ${needN?'on':''}"><i class="dot" style="background:${needN?'#fff':'var(--need)'}"></i><b>${needN}</b> naghihintay sa iyo</span>""",
    """    <span class="count need ${needN+by('failed')?'on':''}"><i class="dot" style="background:${needN?'#fff':'var(--need)'}"></i><b>${needN+by('failed')}</b> alarm</span>""")


rep("const SIT_ACTS={tv:1,chess:1};", "const SIT_ACTS={tv:1,chess:1,read:1};")
rep("const waiting=['billiards','chess'].filter(", "const waiting=['billiards','chess','basketball'].filter(")
rep("""  else opts=m<10?['coffee','cr','stretch','phone','chismis','wander','darts','coffee','billiards']
    :m<30?['tv','billiards','chess','eat','chismis','darts','phone','coffee','stretch','tv']
    :['sleep','sleep','tv','phone','chess','eat','tv'];""",
"""  else opts=m<10?['coffee','cr','stretch','phone','chismis','wander','darts','coffee','billiards','gym','basketball','read']
    :m<30?['tv','billiards','chess','eat','chismis','darts','phone','coffee','gym','basketball','basketball','read','tv']
    :['sleep','sleep','tv','phone','chess','eat','read','tv'];""")
rep("const arrive=()=>{s.pose=act==='sleep'?'sleep':SIT_ACTS[act]?'sit':'stand';};", "const arrive=()=>{s.pose=act==='sleep'?'sleep':SIT_ACTS[act]?'sit':act==='gym'&&s.spot&&s.spot!=='gym2'?'walk':'stand';};")
rep(" sleep:['Zzz','Zzz. Isang minuto pa.']};", """ sleep:['Zzz','Zzz. Isang minuto pa.'],
 read:['Ang ganda ng librong ito.','Nagbabasa ako tungkol sa boiler control.','Tahimik dito, sarap magbasa.'],
 gym:['Isa pang set!','Ang bigat nito.','Cardio muna ako.','Para sa kalusugan.'],
 basketball:['Pasa mo!','Tres!','Ay, sablay!','Rebound!','Depensa, depensa!','Isa pang laro?']};""")
rep("const ICON_IDLE={coffee:'☕',", "const ICON_IDLE={read:'📚',gym:'🏋️',basketball:'🏀',coffee:'☕',")
rep("const REACT={working:", "const REACT={basketball:['🏀','🔥','😤'],gym:['💪','😤'],read:['🤓','📖'],working:")
rep("const IDLE_LBL={coffee:'Nagkakape',", "const IDLE_LBL={read:'Nagbabasa',gym:'Nagdyi-gym',basketball:'Nagba-basketball',coffee:'Nagkakape',")
rep("const IDLE_SENT={coffee:'Nagkakape muna sa coffee bar.',", "const IDLE_SENT={read:'Nagbabasa sa library.',gym:'Nag-eehersisyo sa gym.',basketball:'Naglalaro ng basketball sa half court.',coffee:'Nagkakape muna sa pantry.',")
rep("const IDLE_PLACE={coffee:'Coffee bar',", "const IDLE_PLACE={read:'Library',gym:'Gym',basketball:'Half court',coffee:'Pantry',")
rep("  if(act==='eat'){px(x-7,y0+9,5,3,'#efe6d8');px(x-6,y0+8,3,1,'#c9a27e');}",
    "  if(act==='eat'){px(x-7,y0+9,5,3,'#efe6d8');px(x-6,y0+8,3,1,'#c9a27e');}\n  if(act==='gym'&&e.spot==='gym2'){const w=Math.floor(t*1.6+e.seed)%2;px(x-6,y0+2+w*3,2,2,'#2b3036');px(x+4,y0+2+w*3,2,2,'#2b3036');}")
rep("  const players=sessions.filter(s=>s.idle==='billiards'&&still(s)).length;", r"""  const hoopers=sessions.filter(s=>s.idle==='basketball'&&still(s));
  if(hoopers.length){const ph=(t*.5)%1,a=isoP(hoopers[Math.floor(t*.5)%hoopers.length]);
    if(hoopers.length===1||ph<.55){const k=ph<.55?ph/.55:0;const r=iso(38,14.2,44);const bx=a.x+(r.x-a.x)*k,by=a.y-26+(r.y-(a.y-26))*k-Math.sin(k*Math.PI)*30;ctx.fillStyle='#d9772b';ctx.beginPath();ctx.arc(bx,by,3,0,Math.PI*2);ctx.fill();}
    else{const b=isoP(hoopers[(Math.floor(t*.5)+1)%hoopers.length]);const k=(ph-.55)/.45;const bx=a.x+(b.x-a.x)*k,by=a.y-12+(b.y-a.y)*k-Math.abs(Math.sin(k*Math.PI*3))*14;ctx.fillStyle='#d9772b';ctx.beginPath();ctx.arc(bx,by,3,0,Math.PI*2);ctx.fill();}}
  const players=sessions.filter(s=>s.idle==='billiards'&&still(s)).length;""")
open(dst, 'w', encoding='utf-8').write(h)
print('ok', len(h))

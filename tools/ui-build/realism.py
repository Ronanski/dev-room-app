"""Realistic routines for the plant campus mockup: time of day, real durations, chains, energy, partners, night."""
import sys
p = sys.argv[1]
h = open(p, encoding='utf-8').read()

def rep(old, new, count=1):
    global h
    n = h.count(old)
    assert n == count, f"expected {count} of {old[:90]!r}, found {n}"
    h = h.replace(old, new)

# demo speed switch (the real app runs at 1x)
rep("const FX={icons:true,effects:true,mood:true,timeline:true,walk:true};",
    "const FX={demo:true,icons:true,effects:true,mood:true,timeline:true,walk:true};\nconst SPEED=()=>FX.demo?10:1;")
rep("const FX_LABEL={", "const FX_LABEL={demo:'Demo bilis (10x)',")

# realistic activity choice and durations
rep("""  let opts;
  if(waiting.length&&Math.random()<.75)opts=waiting;
  else opts=m<10?['coffee','cr','stretch','phone','chismis','wander','darts','coffee','billiards','gym','basketball','read']
    :m<30?['tv','billiards','chess','eat','chismis','darts','phone','coffee','gym','basketball','basketball','read','tv']
    :['sleep','sleep','tv','phone','chess','eat','read','tv'];
  let act=pick(opts),spot=null;
  if(act===was&&Math.random()<.7)act=pick(opts);""",
"""  let act=(waiting.length&&Math.random()<.75)?pick(waiting):chooseActivity(s,m,was),spot=null;""")
rep("""  s.idle=act;s.idleUntil=now()+(act==='sleep'?rand(4,8)*MIN:rand(60,150)*1000);""",
    """  s.idle=act;s.actStart=now();s.idleUntil=now()+activityMs(act);""")
rep("const SIT_ACTS={tv:1,chess:1,read:1};", r"""const SIT_ACTS={tv:1,chess:1,read:1};
// Minutes each activity takes in real life; the demo switch speeds everything up 10x.
const DUR={coffee:[2,4],cr:[1,3],eat:[10,20],tv:[8,15],billiards:[6,12],chess:[10,20],basketball:[8,15],gym:[10,20],read:[8,15],darts:[3,6],stretch:[1,2],phone:[2,5],chismis:[3,8],wander:[1,3],sleep:[15,40]};
const hourNow=()=>{const d=new Date();return d.getHours()+d.getMinutes()/60;};
const isLunch=()=>{const x=hourNow();return x>=11.5&&x<13;};
function activityMs(act){let [a,b]=DUR[act]||[2,5];if(act==='eat'&&isLunch()){a=20;b=40;}return rand(a,b)*MIN/SPEED();}
// What a real person would do next: time of day, how tired they are, and what they just did.
function chooseActivity(s,m,was){
  const x=hourNow(),en=typeof energy==='function'?energy(s):.6,w={};
  const add=(k,n)=>{w[k]=(w[k]||0)+n;};
  ['coffee','cr','stretch','phone','chismis','wander','darts','read'].forEach(k=>add(k,2));
  if(m>=10){['tv','billiards','chess','eat','basketball','gym','read'].forEach(k=>add(k,2));}
  if(m>=30){add('sleep',4);add('tv',2);}
  if(isLunch())add('eat',14);
  if(x>=14.75&&x<15.5){add('eat',5);add('coffee',5);}               // merienda
  if(x>=17&&x<20){add('gym',5);add('basketball',6);}                // after work
  if(x>=22||x<6){add('sleep',8);add('tv',2);w.basketball=0;w.gym=0;} // night shift
  if(x>=6&&x<9)add('coffee',6);                                     // morning coffee
  if(en<.35){add('coffee',5);add('sleep',4);w.gym=0;w.basketball=0;}
  if(was==='coffee'||was==='eat')add('cr',6);                       // what goes in comes out
  if(was==='gym'||was==='basketball'){add('cr',5);add('eat',3);add('coffee',2);w.gym=0;w.basketball=0;}
  if(was==='sleep'){add('coffee',6);add('cr',4);w.sleep=0;}
  if(was&&w[was])w[was]*=.25;
  const ks=Object.keys(w).filter(k=>w[k]>0),tot=ks.reduce((a,k)=>a+w[k],0);let r=Math.random()*tot;
  for(const k of ks){r-=w[k];if(r<=0)return k;}return ks[0]||'coffee';
}""")
# linger and desk checks use real minutes too
rep("  if(!s.lingerMs)s.lingerMs=rand(2,5)*MIN;", "  if(!s.lingerMs)s.lingerMs=rand(2,5)*MIN/SPEED();")
rep("if(st==='idle'&&prev!=='idle'){s.idleStart=s.since;s.lingerMs=rand(2,5)*MIN;}", "if(st==='idle'&&prev!=='idle'){s.idleStart=s.since;s.lingerMs=rand(2,5)*MIN/SPEED();}")

# give up on a game when nobody joins
rep("    if(s.status==='idle'&&now()>s.idleUntil&&!s.path.length&&!s.rounds){",
    """    if(s.status==='idle'&&(s.idle==='chess'||s.idle==='billiards')&&!s.path.length&&now()-(s.actStart||0)>90000/SPEED()*3&&!sessions.some(o=>o!==s&&o.idle===s.idle&&!o.path.length)){talk(s,'Walang kalaro. Iba na lang muna.');s.idleUntil=now();}
    if(s.status==='idle'&&now()>s.idleUntil&&!s.path.length&&!s.rounds){""")

# you: lunch at the pantry, breaks with real lengths
rep("      const act=pick(['coffee','cr','stretch','wander','coffee']);release(boss);",
    "      const act=isLunch()?'eat':pick(['coffee','cr','stretch','wander','coffee']);release(boss);")
rep("      const sp=act==='coffee'||act==='cr'?(claim(act,boss)||wanderSpot()):wanderSpot();",
    "      const sp=act==='coffee'||act==='cr'||act==='eat'?(claim(act,boss)||wanderSpot()):wanderSpot();")
rep("      boss.mode='break';boss.idle=act;boss.until=now()+rand(60,120)*1000;",
    "      boss.mode='break';boss.idle=act;boss.until=now()+activityMs(act==='wander'?'wander':act);")
rep("{coffee:['Kape muna ako.','Ang tahimik ng office ngayon.'],cr:['CR lang ako saglit.'],",
    "{eat:['Tanghalian muna.','Ano ang ulam ngayon?'],coffee:['Kape muna ako.','Ang tahimik ng planta ngayon.'],cr:['CR lang ako saglit.'],")
rep("    if(now()-lp>15*MIN){", "    if(now()-lp>15*MIN/SPEED()||isLunch()){")

# guard dozes more at night
rep("    if(Math.random()<.22){guard.state='doze';", "    if(Math.random()<((hourNow()>=22||hourNow()<6)?.45:.18)){guard.state='doze';")

# night: dim the plant, lamps stay on
rep("  if(FX.effects)drawFx(t);", r"""  {const x=hourNow();const dark=x>=20||x<5?.32:x>=18.5?.18:x<6.5?.18:0;
   if(dark){ctx.save();ctx.setTransform(1,0,0,1,0,0);ctx.fillStyle=`rgba(12,18,34,${dark})`;ctx.fillRect(0,0,cv.width,cv.height);ctx.restore();}}
  if(FX.effects)drawFx(t);""")

open(p, 'w', encoding='utf-8').write(h)
print('ok')

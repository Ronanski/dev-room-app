"""Turn the plant mockup back into the app UI: real data source, document skeleton, no sample team, 1x time."""
import sys, re
mock, orig, dst = sys.argv[1], sys.argv[2], sys.argv[3]
h = open(mock, encoding='utf-8').read()
o = open(orig, encoding='utf-8').read()

def rep(old, new, count=1):
    global h
    n = h.count(old)
    assert n == count, f"expected {count} of {old[:90]!r}, found {n}"
    h = h.replace(old, new)

def cut(src, start, end):
    a = src.index(start); b = src.index(end, a); return src[a:b]

# document skeleton and app head
head = o[:o.index('<title>Dev Room</title>')]
rep('<title>Dev Room Plant</title>\n<meta name="claude:viewport-fit-sides" content="cover">',
    '<meta name="theme-color" content="#2a201a">\n<link rel="manifest" href="/manifest.json">\n<link rel="icon" href="/icon.svg">\n<title>Dev Room</title>')
h = head.replace('<meta name="theme-color" content="#2a201a">\n<link rel="manifest" href="/manifest.json">\n<link rel="icon" href="/icon.svg">\n', '') + h
rep('</style>\n\n<div class="stage"', '</style>\n</head>\n<body>\n\n<div class="stage"')
h = h.rstrip('\n') + '\n</body>\n</html>\n'

# real data source: the app's own connection bar and polling
a = h.index('let connBuilt=false;'); b = h.index('let uiBuild=null;')
h = h[:a] + cut(o, 'function setConn(){', 'let uiBuild=null;') + h[b:]

# app wording
rep('<p class="sub">Unit 1 control room · sample na shift · ', '<p class="sub">Unit 1 · live mula sa PC mo · ')
rep("const FX={demo:true,", "const FX={demo:false,")
# the timeline starts when the app first sees a session (no invented history)
rep("""  let x=hashStr(s.sid),t=now()-8*60*MIN;const tl=[];
  while(t<s.since-5*MIN){x=(x*1103515245+12345)>>>0;const st=['working','working','idle','working','needs','idle','working'][x%7];tl.push({st,from:t});t+=(15+x%70)*MIN;}
  tl.push({st:s.status==='idle'&&idleMins(s)>30?'sleep':s.status,from:Math.min(s.since,now())});s.tl=tl;""",
    """  s.tl=[{st:s.status==='idle'&&idleMins(s)>30?'sleep':s.status,from:Math.min(s.since,now())}];""")
for bad in ('SCRIPT=', 'REC={'):
    if bad in h:
        raise SystemExit('leftover sample code: ' + bad)
open(dst, 'w', encoding='utf-8').write(h)
print('ok', len(h))

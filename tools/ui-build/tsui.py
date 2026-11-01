import sys
p=sys.argv[1];h=open(p).read()
def rep(a,b):
    global h
    assert h.count(a)==1,a[:70];h=h.replace(a,b)
rep("settingsPath=j.settings_path||'';lanUrl=j.lan_url||'';","settingsPath=j.settings_path||'';lanUrl=j.lan_url||'';tsUrl=j.ts_url||'';")
rep("let allRecords=[],","let tsUrl='';let allRecords=[],")
rep("""  <p class="hint">Kung magtanong ang Windows Firewall, payagan ang Dev Room sa Private networks.""","""  <p class="hint">${tsUrl?'Naka-on ang Tailscale sa PC na ito, kaya gagana rin ang phone kahit wala ka sa bahay, basta naka-on din ang Tailscale sa phone.':'Para magamit ang phone kahit wala ka sa bahay, i-install ang Tailscale (libre) sa PC at sa phone, mag-login sa parehong account, at i-scan ulit ang QR code.'}</p>
  <p class="hint">Kung magtanong ang Windows Firewall, payagan ang Dev Room sa Private networks.""")
open(p,'w').write(h);print('ok')

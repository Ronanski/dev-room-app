#!/bin/bash
# Build the plant mockup and the app UI from app/ui/index.html (the v0.3.0 UI on main).
set -e
S=$(cd "$(dirname "$0")" && pwd)
OUT=${OUT:-/tmp/devroom-ui-build}
R=$(cd "$S/../.." && pwd)
ORIG=$OUT/orig.html; mkdir -p $OUT
git -C $R show origin/main:app/ui/index.html > $ORIG
python3 $S/mock04.py $ORIG $OUT/preview.html
sed -i "s/<p class=\"sub\">Live mula sa PC mo · /<p class=\"sub\">Sample na office · /" $OUT/preview.html
python3 $S/cr_iso2.py $OUT/preview.html $OUT/plant.html
sed -i 's#<title>Dev Room Control Room Iso</title>#<title>Dev Room Plant</title>#; s#Isometric na mockup. Sample na shift na may kunwaring events kada 20 segundo. Nasa Alarms tab ang mga naghihintay sa iyo, at may ACK ang bawat isa.#Sample na shift na may kunwaring events kada 20 segundo. Sumusunod sa totoong oras ang buhay sa planta: tanghalian, merienda, gym pagkatapos ng trabaho, at tulog sa gabi. Nasa Alarms tab ang mga naghihintay sa iyo.#' $OUT/plant.html
python3 $S/realism.py $OUT/plant.html
python3 $S/npcs.py $OUT/plant.html
sed -i "s/log('HR',/log('Safety',/g" $OUT/plant.html
python3 $S/bubbles.py $OUT/plant.html
python3 $S/unmock.py $OUT/plant.html $ORIG $OUT/app.html
for f in plant app; do python3 -c "
import re,sys;h=open('$OUT/$f.html').read();s=re.findall(r'<script>(.*?)</script>',h,re.S);open('$OUT/$f.js','w').write('\n'.join(s))"; node --check $OUT/$f.js; echo "$f js ok"; done
python3 $S/tsui.py $OUT/app.html

"""Independent JPL root check for Neptune's 2025 Aries ingress (TT)."""
import json
from fetch_jpl import fetch, ROOT

a,b=2460750.5,2460770.5
history=[]
for iteration in range(5):
    params,raw,values=fetch(899,[a,b])
    (ROOT/f'neptune_ingress_{iteration}.json').write_bytes(raw)
    angles=[(row[1]+180)%360-180 for row in values]
    root=a-angles[0]*(b-a)/(angles[1]-angles[0])
    history.append(dict(parameters=params,values=values,root_jd_tt=root))
    print(iteration,root,angles,flush=True)
    a,b=root-1/86400,root+1/86400
(ROOT/'neptune_ingress.json').write_text(json.dumps(history,indent=2)+'\n')

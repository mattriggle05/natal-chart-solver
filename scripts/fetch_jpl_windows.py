"""Build 12 fixed search references using only JPL positions, never Rust output.
Run --verify to check retained requests and regenerate expected windows offline.
"""
import argparse,csv,hashlib,json,re
import urllib.error
from pathlib import Path
from fetch_jpl import fetch
ROOT=Path(__file__).resolve().parents[1]/'fixtures/jpl/windows'
# name, start TT, end TT, [(feature, arc start, arc span)], time budget seconds
CASES=[
 ('sun_aries',2460380.5,2460440.5,[(10,0,30)],30),
 ('moon_wrapped',2460400.5,2460410.5,[(11,350,20)],60),
 ('moon_phase',2460400.5,2460440.5,[(16,0,30)],60),
 ('mercury_reentry',2460380.5,2460450.5,[(0,20,5)],120),
 ('mercury_station_window',2460380.5,2460450.5,[(0,27,1)],180),
 ('sun_and_moon',2460380.5,2460440.5,[(10,0,30),(11,0,30)],60),
 ('clipped_both',2460405.5,2460406.5,[(10,0,30)],0),
 ('clipped_start',2460405.5,2460440.5,[(10,0,30)],30),
 ('clipped_end',2460380.5,2460405.5,[(10,0,30)],30),
 ('empty_combination',2460380.5,2460440.5,[(10,0,30),(0,180,30)],30),
 ('neptune_aries',2460750.5,2460770.5,[(7,0,30)],1200),
 ('sun_subminute',2460380.5,2460440.5,[(10,10,0.0002)],30),
]
COMMAND={0:199,7:899,10:10,11:301}
STEP=0.125 # three-hour independent reference sampling

def difference(a,b): return (a-b+180)%360-180

def generate(offline=False):
 ROOT.mkdir(parents=True,exist_ok=True)
 manifest=[]; cache={}; counter=0
 old_manifest=json.loads((ROOT/'manifest.json').read_text()) if offline else None
 def positions(feature,dates):
  nonlocal counter
  dates=sorted(set(dates))
  missing=[jd for jd in dates if (feature,jd) not in cache]
  for offset in range(0,len(missing),40):
   requested=missing[offset:offset+40]; filename=f'{counter:03d}.json'
   if offline:
    entry=old_manifest[counter];raw=(ROOT/filename).read_bytes()
    if entry['file']!=filename or entry['feature']!=feature or entry['dates']!=requested or hashlib.sha256(raw).hexdigest()!=entry['sha256']: raise ValueError('Reference manifest mismatch')
    result=json.loads(raw)['result']; records=list(csv.reader(result.split('$$SOE')[1].split('$$EOE')[0].strip().splitlines()))
    values=[(float(row[0]),float(row[3]),float(row[4])) for row in records]
   else:
    params=dict(format='json', COMMAND=f"'{COMMAND[feature]}'", CENTER="'500@399'", EPHEM_TYPE="'OBSERVER'", TIME_TYPE="'TT'", TLIST="'"+','.join(map(str,requested))+"'", QUANTITIES="'31'", CSV_FORMAT="'YES'", EXTRA_PREC="'YES'", OBJ_DATA="'NO'", CAL_FORMAT="'JD'")
    cached=False
    if (ROOT/filename).exists():
     raw=(ROOT/filename).read_bytes(); result=json.loads(raw)['result']
     records=list(csv.reader(result.split('$$SOE')[1].split('$$EOE')[0].strip().splitlines()))
     values=[(float(row[0]),float(row[3]),float(row[4])) for row in records]
     cached=bool(re.search(r'Target body name:.*\('+str(COMMAND[feature])+r'\)',result)) and len(values)==len(requested) and all(abs(row[0]-jd)<1e-7 for row,jd in zip(values,requested))
    if not cached:
     for attempt in range(3):
      try:
       params,raw,values=fetch(COMMAND[feature],requested);break
      except (urllib.error.URLError,TimeoutError):
       if attempt==2:raise
     (ROOT/filename).write_bytes(raw)
    entry=dict(file=filename,feature=feature,dates=requested,parameters=params,sha256=hashlib.sha256(raw).hexdigest())
    print('Cached' if cached else 'JPL request',counter,feature,len(requested),flush=True)
   required=dict(COMMAND=f"'{COMMAND[feature]}'",CENTER="'500@399'",EPHEM_TYPE="'OBSERVER'",TIME_TYPE="'TT'",QUANTITIES="'31'",CSV_FORMAT="'YES'",EXTRA_PREC="'YES'",CAL_FORMAT="'JD'",TLIST="'"+','.join(map(str,requested))+"'")
   if any(entry['parameters'].get(key)!=value for key,value in required.items()):raise ValueError('Reference request conventions changed')
   result=json.loads(raw)['result']
   if ' TT ' not in result or 'AIRLESS' not in result or 'GEOCENTRIC' not in result:raise ValueError('Unexpected reference frame/time')
   if len(values)!=len(requested): raise ValueError('Missing returned positions')
   for jd,row in zip(requested,values):
    if abs(jd-row[0])>1e-7: raise ValueError('Returned epoch differs')
    cache[feature,jd]=row[1]
   manifest.append(entry); counter+=1
   if not offline:(ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
  return [cache[feature,jd] for jd in dates]
 def angles(feature,dates):
  if feature==16:
   moon=positions(11,dates);sun=positions(10,dates)
   return [(m-s)%360 for m,s in zip(moon,sun)]
  return positions(feature,dates)
 # Per-feature grids and crossings shared across cases.
 constraints={}
 for _,start,end,arcs,_ in CASES:
  for feature,angle,span in arcs:
   key=(start,end,feature,angle,span)
   if key in constraints:continue
   dates=[start+i*STEP for i in range(round((end-start)/STEP)+1)]
   values=angles(feature,dates)
   crossings=[]
   for t0,t1,v0,v1 in zip(dates,dates[1:],values,values[1:]):
    delta=difference(v1,v0)
    for boundary in [angle,(angle+span)%360]:
     displacement=difference(boundary,v0)
     if delta and 0 < displacement/delta <= 1:
      crossings.append([t0,t1,difference(v0,boundary),difference(v1,boundary),boundary])
   # Bracket-preserving secant refinements, batched across all active roots.
   exact={}
   for iteration in range(8):
    active=[i for i in range(len(crossings)) if i not in exact]
    trials={i:crossings[i][0]-crossings[i][2]*(crossings[i][1]-crossings[i][0])/(crossings[i][3]-crossings[i][2]) for i in active}
    if not trials:break
    answers=dict(zip(sorted(set(trials.values())),angles(feature,list(trials.values()))))
    for i,t in trials.items():
     root=crossings[i];error=difference(answers[t],root[4])
     if error==0:exact[i]=t
     elif root[2]*error>0:root[0],root[2]=t,error
     else:root[1],root[3]=t,error
   roots=[exact[i] if i in exact else a-fa*(b-a)/(fb-fa) for i,(a,b,fa,fb,_) in enumerate(crossings)]
   # Verify every resulting boundary against direct JPL samples one second
   # either side, rather than trusting interpolation convergence alone.
   probes=[t+sign/86400 for t in roots for sign in [-1,1]]
   answers=dict(zip(sorted(set(probes)),angles(feature,probes))) if probes else {}
   for t,root in zip(roots,crossings):
    if difference(answers[t-1/86400],root[4])*difference(answers[t+1/86400],root[4])>0:
     raise ValueError('Reference root failed independent one-second bracket check')
   edges=[start]+sorted(roots)+[end]
   # Independent midpoint evaluations confirm each whole candidate window.
   mids=[(a+b)/2 for a,b in zip(edges,edges[1:])]
   midpoint_angles=dict(zip(sorted(set(mids)),angles(feature,mids)))
   windows=[(a,b) for a,b,m in zip(edges,edges[1:],mids) if (midpoint_angles[m]-angle)%360<span]
   constraints[key]=windows
 output=[]
 for name,start,end,arcs,budget in CASES:
  windows=[(start,end)]
  for feature,angle,span in arcs:
   windows=[(max(a,c),min(b,d)) for a,b in windows for c,d in constraints[start,end,feature,angle,span] if max(a,c)<min(b,d)]
  windows.sort()
  if name=='mercury_station_window':
   dates=[start+i*STEP for i in range(round((end-start)/STEP)+1)]
   values=angles(0,dates)
   peaks=[dates[i] for i in range(1,len(dates)-1) if values[i]>values[i-1] and values[i]>values[i+1] and 27<values[i]<28]
   if not any(a<t<b for a,b in windows for t in peaks):raise ValueError('Station case does not contain an independently sampled turning point')

  output.append('|'.join([name,str(start),str(end),','.join(str(a[0]) for a in arcs),','.join(str(a[1]) for a in arcs),','.join(str(a[2]) for a in arcs),str(budget),','.join(str(v) for w in windows for v in w)]))
 text='\n'.join(output)+'\n'
 if offline:
  if counter!=len(old_manifest) or text!=(ROOT/'cases.txt').read_text():raise ValueError('Offline regeneration differs')
  print('Verified',counter,'raw responses and',len(CASES),'reference searches offline')
 else:
  (ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
  (ROOT/'cases.txt').write_text(text)
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--verify',action='store_true')
 generate(parser.parse_args().verify)

"""Fetch geometric ICRF/ecliptic J2000 vectors for isolated model diagnosis."""
import csv,io,json,urllib.parse,urllib.request
from pathlib import Path
from fetch_jpl import DATES
ROOT=Path(__file__).resolve().parents[1]/'fixtures/jpl/diagnosis'
ROOT.mkdir(exist_ok=True)
T=2460764.999763406
DATES=sorted(set(DATES+[T+i/100 for i in range(-30,3)]+[T-0.005,T-0.175]))
rows=[]
for body in [399,899,8,10]:
 p=dict(format='json',COMMAND=f"'{body}'",CENTER="'500@0'",EPHEM_TYPE="'VECTORS'",TIME_TYPE="'TDB'",TLIST="'"+','.join(map(str,DATES))+"'",REF_PLANE="'ECLIPTIC'",REF_SYSTEM="'ICRF'",OUT_UNITS="'AU-D'",VEC_TABLE="'2'",VEC_CORR="'NONE'",CSV_FORMAT="'YES'",OBJ_DATA="'NO'")
 with urllib.request.urlopen('https://ssd.jpl.nasa.gov/api/horizons.api?'+urllib.parse.urlencode(p),timeout=60) as r: raw=r.read()
 (ROOT/f'{body}.json').write_bytes(raw)
 result=json.loads(raw)['result']
 data=list(csv.reader(io.StringIO(result.split('$$SOE')[1].split('$$EOE')[0].strip())))
 rows.extend([body,float(row[0]),*map(float,row[2:8])] for row in data)
 print(body,len(data),flush=True)
 (ROOT/f'{body}_query.json').write_text(json.dumps(p,indent=2))
with (ROOT/'vectors.csv').open('w') as f:
 w=csv.writer(f);w.writerow(['body','jd_tdb','x','y','z','vx','vy','vz']);w.writerows(rows)

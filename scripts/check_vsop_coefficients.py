"""Evaluate original Neptune VSOP87D coefficients independently of both Rust crates.
Pass a downloaded VSOP87D.nep file; original distribution mirror:
https://raw.githubusercontent.com/ctdk/vsop87/master/VSOP87D.nep
"""
import csv,hashlib,json,math,re,sys
from pathlib import Path
from fetch_jpl import DATES
raw=Path(sys.argv[1]).read_bytes()
terms=[]
for line in raw.decode().splitlines():
 if 'VSOP87 VERSION' in line:
  variable=int(re.search(r'VARIABLE\s+(\d)',line)[1]);power=int(re.search(r'\*T\*\*(\d)',line)[1])
 elif variable==1:
  a,b,c=map(float,line.split()[-3:]);terms.append((power,a,b,c))
root=Path(__file__).resolve().parents[1]/'fixtures/jpl/diagnosis'
with (root/'original_neptune.csv').open('w') as f:
 w=csv.writer(f);w.writerow(['jd','heliocentric_longitude_radians'])
 for jd in sorted(set(DATES+[2460764.999763406])):
  t=(jd-2451545)/365250
  angle=math.fsum(a*math.cos(b+c*t)*t**p for p,a,b,c in terms)%(2*math.pi)
  w.writerow([jd,angle])
(root/'original_neptune_source.json').write_text(json.dumps(dict(url='https://raw.githubusercontent.com/ctdk/vsop87/master/VSOP87D.nep',sha256=hashlib.sha256(raw).hexdigest(),longitude_terms=len(terms)),indent=2)+'\n')
print('Evaluated',len(terms),'longitude terms')

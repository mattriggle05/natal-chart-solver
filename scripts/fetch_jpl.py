"""Regenerate offline apparent longitude fixtures from the JPL Horizons API."""
import csv
import hashlib
import io
import json
from pathlib import Path
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1] / 'fixtures' / 'jpl'
# TT dates spanning the entire supported range; 2024 dates also sample retrograde seasons.
DATES = [2415020.5,2424151.5,2433282.5,2442413.5,2451545.0,2460402.5,2460417.5,2460432.5,2460676.5,2469807.5,2478938.5,2488069.5]
BODIES = [(0,199),(1,299),(3,499),(4,599),(5,699),(6,799),(7,899),(10,10),(11,301)]

def fetch(command, dates):
    params = dict(format='json', COMMAND=f"'{command}'", CENTER="'500@399'", EPHEM_TYPE="'OBSERVER'", TIME_TYPE="'TT'", TLIST="'"+','.join(map(str,dates))+"'", QUANTITIES="'31'", CSV_FORMAT="'YES'", EXTRA_PREC="'YES'", OBJ_DATA="'NO'", CAL_FORMAT="'JD'")
    url = 'https://ssd.jpl.nasa.gov/api/horizons.api?' + urllib.parse.urlencode(params)
    with urllib.request.urlopen(url, timeout=60) as response:
        raw = response.read()
    data = json.loads(raw)
    result = data.get('result', '')
    if '$$SOE' not in result:
        raise RuntimeError(data)
    records = list(csv.reader(io.StringIO(result.split('$$SOE')[1].split('$$EOE')[0].strip())))
    values = [(float(row[0]),float(row[3]),float(row[4])) for row in records]
    if len(values) != len(dates):
        raise RuntimeError('Unexpected record count')
    return params, raw, values

if __name__ == '__main__':
    ROOT.mkdir(parents=True, exist_ok=True)
    rows=[]
    manifest=[]
    for feature, command in BODIES:
        params, raw, values=fetch(command,DATES)
        filename=f'{command}.json'
        (ROOT/filename).write_bytes(raw)
        manifest.append(dict(file=filename, parameters=params,sha256=hashlib.sha256(raw).hexdigest()))
        rows.extend((feature,*value) for value in values)
        print(command, len(values), flush=True)
    with (ROOT/'positions.csv').open('w') as output:
        writer=csv.writer(output);writer.writerow(['feature','jd_tt','longitude_deg','latitude_deg']);writer.writerows(rows)
    (ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')

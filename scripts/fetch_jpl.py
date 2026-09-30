"""Regenerate offline apparent longitude fixtures from the JPL Horizons API."""
import argparse
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

def verify_fixtures():
    """Offline provenance check: manifests, raw responses, and CSV must agree."""
    manifest = json.loads((ROOT/'manifest.json').read_text())
    if [entry['file'] for entry in manifest] != [f'{command}.json' for _,command in BODIES]:
        raise ValueError('Manifest body coverage changed')
    expected_rows = []
    for (feature,command),entry in zip(BODIES,manifest):
        raw = (ROOT/entry['file']).read_bytes()
        if hashlib.sha256(raw).hexdigest() != entry['sha256']:
            raise ValueError(f"Reference hash mismatch: {entry['file']}")
        parameters = entry['parameters']
        required = dict(COMMAND=f"'{command}'", CENTER="'500@399'", EPHEM_TYPE="'OBSERVER'", TIME_TYPE="'TT'", QUANTITIES="'31'", CSV_FORMAT="'YES'", EXTRA_PREC="'YES'", CAL_FORMAT="'JD'")
        if any(parameters.get(key) != value for key,value in required.items()):
            raise ValueError('Reference coordinate/time convention changed')
        dates = [float(value) for value in parameters['TLIST'].strip("'").split(',')]
        if dates != DATES:
            raise ValueError('Reference epoch coverage changed')
        result = json.loads(raw)['result']
        if ' TT ' not in result or 'AIRLESS' not in result:
            raise ValueError('Reference response time/refraction convention changed')
        records = list(csv.reader(io.StringIO(result.split('$$SOE')[1].split('$$EOE')[0].strip())))
        values = [(float(row[0]),float(row[3]),float(row[4])) for row in records]
        if [value[0] for value in values] != DATES:
            raise ValueError('Returned epochs differ from requested epochs')
        expected_rows.extend((feature,*value) for value in values)
    with (ROOT/'positions.csv').open() as source:
        reader = csv.reader(source)
        if next(reader) != ['feature','jd_tt','longitude_deg','latitude_deg']:
            raise ValueError('Unexpected CSV header')
        actual_rows = [(int(row[0]),*map(float,row[1:])) for row in reader]
    if actual_rows != expected_rows:
        raise ValueError('Position CSV does not reproduce retained JPL responses')
    print(f'Verified {len(manifest)} reference hashes and {len(actual_rows)} positions offline')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify', action='store_true', help='Verify retained fixtures without network access')
    args = parser.parse_args()
    if args.verify:
        verify_fixtures()
        raise SystemExit(0)
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
    verify_fixtures()

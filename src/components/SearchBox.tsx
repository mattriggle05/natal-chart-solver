import { useDataSearch } from '../hooks/useDateSearch';
import styles from './SearchBox.module.css';

function jdToDate(jd: number): string {
    const z = Math.floor(jd + 0.5);
    const a = Math.floor((z - 1867216.25) / 36524.25);
    const b = z + 1 + a - Math.floor(a / 4);
    const c = b + 1524;
    const d = Math.floor((c - 122.1) / 365.25);
    const e = Math.floor(365.25 * d);
    const f = Math.floor((c - e) / 30.6001);

    const day   = c - e - Math.floor(30.6001 * f);
    const month = f < 14 ? f - 1 : f - 13;
    const year  = month > 2 ? d - 4716 : d - 4715;

    return `${month.toString().padStart(2, '0')}/${day.toString().padStart(2, '0')}/${year}`;
}

function formatResults(raw: Float64Array): string {
    const windows: string[] = [];
    for (let i = 0; i + 1 < raw.length; i += 2) {
        windows.push(`${jdToDate(raw[i])} - ${jdToDate(raw[i + 1])}`);
    }
    return windows.join(', ');
}

function SearchBox() { 
    const { search, results } = useDataSearch();

    const startSearch2 = () => {
        const jde1 = new Date('2005-01-01').getTime() / 86400000.0 + 2440587.5;
        const jde2 = new Date('2006-01-01').getTime() / 86400000.0 + 2440587.5;
        console.log('calling search')
        search({
            startJd: jde1,
            endJd: jde2,
            featureIds: [10],
            featureSigns: [5]
        });
    }

    return <> 
        <p className={styles.result}>{formatResults(results)}</p>
        <button onClick={startSearch2}>Search2</button>
    </>;
}

export default SearchBox;
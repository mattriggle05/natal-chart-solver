import { civilDateToJdTt, jdTtToCivil } from '../utils/time';
import type { JulianDateTt } from '../utils/time';
import { useDataSearch } from '../hooks/useDateSearch';
import { Feature } from '../types/features';
import styles from './SearchBox.module.css';

function jdToDate(jd: number): string {
    const civil = jdTtToCivil(jd as JulianDateTt);
    return `${new Date(civil.unixMs).toISOString().slice(0, 10)} ${civil.scale}`;
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

    const startSearch = () => {
        const jde1 = civilDateToJdTt('2005-01-01');
        const jde2 = civilDateToJdTt('2006-01-01');
        console.log('calling search')
        search({
            startJdTt: jde1,
            endJdTt: jde2,
            featureIds: [Feature.Sun],
            angleStarts: [5 * 30],
            angleSpans: [30]
        });
    }

    return <> 
        <p className={styles.result}>{formatResults(results)}</p>
        <button onClick={startSearch}>Search</button>
    </>;
}

export default SearchBox;

import { civilDateToJdTt, formatWindowMinutes } from '../utils/time';
import type { JulianDateTt } from '../utils/time';
import { useDataSearch } from '../hooks/useDateSearch';
import { Feature } from '../types/features';
import styles from './SearchBox.module.css';

function formatResults(raw: Float64Array): string {
    const windows: string[] = [];
    for (let i = 0; i + 1 < raw.length; i += 2) {
        windows.push(formatWindowMinutes(raw[i] as JulianDateTt, raw[i + 1] as JulianDateTt));
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

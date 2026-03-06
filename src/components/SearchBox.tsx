// import { useState, useEffect } from 'react';
// import init, { search2 } from '@wasm/natal_chart_solver';
// import styles from './SolarSystem.module.css';
// import clsx from 'clsx';
import { useDataSearch } from '../hooks/useDateSearch';
import styles from './SearchBox.module.css';

function SearchBox() { 
    const { search, results } = useDataSearch();

    const startSearch2 = () => {
        const jde1 = new Date('2025-01-01').getTime() / 86400000.0 + 2440587.5;
        const jde2 = new Date('2027-01-01').getTime() / 86400000.0 + 2440587.5;
        console.log('calling search')

        search({
            startJd: jde1,
            endJd: jde2,
            featureIds: [10],
            featureSigns: [5]
        });
    }

    return <> 
        <p className={styles.result} >{results.join(", ")}</p>
        <button onClick={startSearch2}>Search2</button>
    </>;
}

export default SearchBox;
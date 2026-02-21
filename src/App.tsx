import { useState } from 'react';
import SolarSystem from './components/SolarSystem';
import styles from './App.module.css';
import { useDataSearch } from './hooks/useDateSearch';
// import clsx from 'clsx';


function App() {

    const [currDate, setCurrDate] = useState<string>('2026-01-01');
    const { search, results } = useDataSearch();

    const startSearch = () => {
        const jde1 = new Date('1927-01-01').getTime() / 86400000.0 + 2440587.5;
        const jde2 = new Date('2027-01-01').getTime() / 86400000.0 + 2440587.5;
        console.log('calling search')
        search({
            startJd: jde1,
            endJd: jde2,
            featureIds: [10, 0, 1, 3, 4, 5, 6, 7],
            featureSigns: [5, 6, 7, 3, 4, 0, 2, 0]
        });
    }

    return (
        <>
            <div className={styles.description}>
                <h1>Coming soon...</h1>
            </div>

            <div className={ styles.container }>
                <SolarSystem date={new Date(currDate)} />
            </div>

            <input type="date" className={styles.dateInput} value={currDate} onChange={e => setCurrDate(e.target.value)} />

            <button onClick={startSearch}>Search</button>
            
            <p>{ results }</p>
        </>
    );
}

export default App;

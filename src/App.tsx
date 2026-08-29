import { useState } from 'react';
import SolarSystem from './components/SolarSystem';
import SearchBox from './components/SearchBox';
import styles from './App.module.css';


function App() {
    const [currDate, setCurrDate] = useState<string>('2026-01-01');

    return (
        <>
            <div className={styles.description}>
                <h1>Coming soon...</h1>
            </div>
            <div className={ styles.container }>
                <SolarSystem date={new Date(currDate)} />
            </div>
            <input type="date" className={styles.dateInput} value={currDate} onChange={e => setCurrDate(e.target.value)} />

            <br />
            
            <SearchBox />
        </>
    );
}

export default App;

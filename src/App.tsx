import { useState } from 'react';
import SolarSystem from './components/SolarSystem';
import styles from './App.module.css';

export default function App() {
    const [currDate, setCurrDate] = useState('2026-01-01');
    return <>
        <div className={styles.description}><h1>Coming soon...</h1></div>
        <div className={styles.container}><SolarSystem date={new Date(currDate)} /></div>
        <input aria-label="Solar system date" type="date" className={styles.dateInput} value={currDate} onChange={event => { if (event.target.value) setCurrDate(event.target.value); }} />
        <div className={styles.previewLink}>
            <button type="button" onClick={() => { window.location.href = `${import.meta.env.BASE_URL}preview/`; }}>Check out the preview...</button>
        </div>
    </>;
}

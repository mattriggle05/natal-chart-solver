import { useState } from 'react';
import SolarSystem from './SolarSystem';
import styles from './App.module.css';
import clsx from 'clsx';


function App() {

  const [currDate, setCurrDate] = useState<string>('2026-01-01');

  return (
    <>
      <div className={styles.description}>
        <h1>Coming soon...</h1>
      </div>

      <div className={ styles.container }>
        <SolarSystem date={ new Date(currDate) } />
      </div>
      
      <input type="date" value={currDate} onChange={e => setCurrDate(e.target.value)} />
    </>
  );
}

export default App;

import { useState, useEffect } from 'react';
import SolarSystem from './SolarSystem';
import styles from './App.module.css';
import init, { search } from '@wasm/natal_chart_solver';
// import clsx from 'clsx';


function App() {
  const [currDate, setCurrDate] = useState<string>('2026-01-01');
  const [calculationResult, setCalculationResult] = useState<Float64Array>(new Float64Array([0,0,0,0,0,0,0,0]));

  useEffect(() => {
    init()
      .then(() => {
        const jde1 = new Date('2026-01-01').getTime() / 86400000.0 + 2440587.5;
        const jde2 = new Date('2027-01-01').getTime() / 86400000.0 + 2440587.5;
        const result = search(jde1, jde2, new Uint8Array([10, 0, 1, 3, 4, 5, 6, 7]), new Uint8Array([5, 6, 7, 3, 4, 0, 2, 0]));

        if (result) {
          setCalculationResult(result);
          console.log(result);
        } else { 
          console.log('no result')
        }
      })
      .catch((e: unknown ) => console.log(`an unknown error`));
  }, [currDate]);

  return (
    <>
      <div className={styles.description}>
        <h1>Coming soon...</h1>
      </div>

      <div className={ styles.container }>
        <SolarSystem date={new Date(currDate)} />
      </div>

      <input type="date" className={styles.dateInput} value={currDate} onChange={e => setCurrDate(e.target.value)} />

      <p>{ calculationResult }</p>
    </>
  );
}

export default App;

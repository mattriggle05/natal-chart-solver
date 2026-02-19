import { useState, useEffect } from 'react';
import SolarSystem from './SolarSystem';
import styles from './App.module.css';
import init, { geocentric_longitudes_at_jde } from '@wasm/natal_chart_solver';
// import clsx from 'clsx';


function App() {
  const [currDate, setCurrDate] = useState<string>('2026-01-01');
  const [wasmResult, setWasmResult] = useState<string>('loading...');

  useEffect(() => {
    init()
      .then(() => {
        const jde = new Date(currDate).getTime() / 86400000.0 + 2440587.5;
        const result = geocentric_longitudes_at_jde(jde, new Uint8Array([0, 1, 3, 4, 5, 6, 7]));

        if (result) {
          setWasmResult(`${result}`);
        } else { 
          setWasmResult(`An error occurred`);
        }
      })
      .catch((e: unknown ) => setWasmResult(`WASM error: ${e}`));
  }, [currDate]);

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

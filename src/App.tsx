import { useState, useEffect } from 'react';
import SolarSystem from './SolarSystem';
import styles from './App.module.css';
import init, { neptune_longitude } from '@wasm/natal_chart_solver';
// import clsx from 'clsx';


function App() {
  const [currDate, setCurrDate] = useState<string>('2026-01-01');
  const [wasmResult, setWasmResult] = useState<string>('loading...');

  useEffect(() => {
    init()
      .then(() => {
        const longitude = neptune_longitude(2451545.0);
        setWasmResult(`${longitude.toFixed(4)}°`);
      })
      .catch((e: unknown ) => setWasmResult(`WASM error: ${e}`));
  }, []);

  return (
    <>
      <div className={styles.description}>
        <h1>Coming soon...</h1>
      </div>

      <div className={ styles.container }>
        <SolarSystem date={ new Date(currDate) } />
      </div>
      
      <input type="date" value={currDate} onChange={e => setCurrDate(e.target.value)} />

      <p>{ wasmResult }</p>
    </>
  );
}

export default App;

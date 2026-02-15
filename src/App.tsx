import { useState, useMemo } from 'react';
import SolarSystem from './SolarSystem';
import './App.css';


function App() {

  const [currDate, setCurrDate] = useState<string>('2026-01-01');

  return (
    <>
      <div className='description'>
        <h1>Coming soon...</h1>
      </div>

      <div className="system-container">
        <SolarSystem date={ currDate } />
      </div>
      
      <input type="date" value={currDate} onChange={e => setCurrDate(e.target.value)} />
    </>
  );
}

export default App;

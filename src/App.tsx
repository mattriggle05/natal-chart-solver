import { useState, useMemo } from 'react';
import SolarSystem from './SolarSystem';
import './App.css';
import { Ecliptic, Body, GeoVector } from 'astronomy-engine';


function App() {

  const [currBody, setCurrBody] = useState<Body>(Body.Neptune);
  const [currDate, setCurrDate] = useState<string>('2026-01-01');

  var result = useMemo(() => Ecliptic(GeoVector(currBody, new Date(currDate), true)).elon, [currBody,currDate])

  return (
    <>
      <div className='description'>
        <h1>Coming soon...</h1>
      </div>

      <div className="system-container">
        <SolarSystem />
      </div>
      

      <select value={currBody} onChange={e => setCurrBody(e.target.value as Body)}>
        {Object.values(Body).map(x => <option key={ x } value ={ x }>{ x }</option>)}
      </select>

      <input type="date" value={currDate} onChange={e => setCurrDate(e.target.value)} />

      <p style={{color: 'white'}}>{result}</p>
    </>
  );
}

export default App;

import { useState, useMemo } from 'react';
import SolarSystem from './SolarSystem';
import './App.css';
import { EclipticLongitude, Body, FlexibleDateTime } from 'astronomy-engine';


function App() {

  const [currBody, setCurrBody] = useState<Body>(Body.Neptune);
  const [currDate, setCurrDate] = useState<string>('2026-01-01');

  var result = useMemo(() =>EclipticLongitude(currBody, new Date(currDate)), [currBody,currDate])

  return (
    <>
      <div className='description'>
        <h1>Coming soon...</h1>
      </div>

      <SolarSystem />

      <select value={currBody} onChange={e => setCurrBody(e.target.value as Body)}>
        {Object.values(Body).map(x => <option>{x}</option>)}
      </select>

      <input type="date" value={currDate} onChange={e => setCurrDate(e.target.value)} />

      <p>{result}</p>
    </>
  );
}

export default App;

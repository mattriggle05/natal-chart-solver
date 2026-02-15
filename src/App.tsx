// import React from 'react';
import SolarSystem from './SolarSystem';
import './App.css';
import { EclipticLongitude, Body, FlexibleDateTime } from 'astronomy-engine';

function App() {

  var result: number = EclipticLongitude(Body.Neptune, new Date(2026,1,1))

  return (
    <>
      <div className='description'>
        <h1>Coming soon...</h1>
      </div>

      <SolarSystem />

      <select>
        {Object.values(Body).map(x => <option>{x}</option>)}
      </select>

      <p>{ result }</p>
    </>
  );
}

export default App;

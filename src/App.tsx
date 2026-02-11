import React from 'react';
import './App.css';

function App() {
  return (
    <div>
      <h1>Natal Chart Solver (WIP)</h1>
      <p>This is a React App built with TypeScript and deployed to Github pages.</p>
      <p>Most people know their "Star sign" or "Birth sign," for example, being born in the beginning of september makes me a Virgo. What most people don't know is that they in fact have a number of these signs, one for the sun, the moon, each of the planets and some other important astronomical features. Each of these features, at any point in time, is some portion of the way through the ecliptic, which is the path that the sun traces through the sky. The ecliptic passes over different constellations, giving the signs their names. We assign these signs to ourselves by where these astronomical features were on the date and time of our birth.</p>
      <p>Natal Chart Solver takes in information about one's star signs and attempts to use them to calculate a persons birth date and perhaps birth time. It does this by taking in a multitude of signs and calculating a range dates when the planets would be aligned correctly along the ecliptic to produce such a result. With enough data points (i.e. the signs of multiple planets) we can narrow this range down to hopefully within a single day.</p>
    </div>
  );
}

export default App;

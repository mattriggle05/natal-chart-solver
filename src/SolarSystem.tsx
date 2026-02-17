import { useState, useEffect } from 'react';
import './SolarSystem.css';
import { Body, EclipticLongitude, FlexibleDateTime } from 'astronomy-engine';


function SolarSystem({ date }: {date: FlexibleDateTime}) { 
    const planets: Body[] = [Body.Mercury, Body.Venus, Body.Earth, Body.Mars, Body.Jupiter, Body.Saturn, Body.Uranus, Body.Neptune];
    const [planetPositions, setPlanetPositions] = useState(Object.fromEntries(planets.map(p => [p, 0])));

    useEffect(() => {
        setPlanetPositions(
            Object.fromEntries(
                planets.map(p => [p, EclipticLongitude(Body[p], date)])
            )
        );
    }, [date]);

    return <div className="system">
        <div className="sun"></div>
        {Object.keys(planetPositions).map(planet =>
            <div key={planet} className={`orbit ${planet}`} style={{transform: `translate(-50%,-50%) rotate(${planetPositions[planet]}deg)`}}>
                <div className={`planet ${planet}`} />
            </div>
        )}
    </div>;
}

export default SolarSystem;
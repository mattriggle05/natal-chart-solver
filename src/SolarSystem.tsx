import { useState, useEffect } from 'react';
import { Body, EclipticLongitude, FlexibleDateTime } from 'astronomy-engine';
import styles from './SolarSystem.module.css';
import clsx from 'clsx';

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

    return <div className={styles.system}>
        <div className={styles.sun}></div>
        {Object.keys(planetPositions).map(p =>
            <div key={p} className={clsx(styles.orbit, styles[p])} style={{transform: `translate(-50%,-50%) rotate(${planetPositions[p]}deg)`}}>
                <div className={clsx(styles.planet, styles[p])} />
            </div>
        )}
    </div>;
}

export default SolarSystem;
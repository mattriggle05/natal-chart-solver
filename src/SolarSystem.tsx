import { useState, useMemo } from 'react';
import './SolarSystem.css';
import { Body, EclipticLongitude } from 'astronomy-engine';


function SolarSystem({ date }: {date: string}) { 
    
    const planets = ['Mercury', 'Venus', 'Earth', 'Mars', 'Jupiter', 'Saturn', 'Uranus', 'Neptune'] as const;

    const [planetPositions, setPlanetPositions] = useState(
        Object.fromEntries(planets.map(p => [p, 0]))
    );

    useMemo(() => {
        const dateObj = new Date(date);
        setPlanetPositions(
            Object.fromEntries(
                planets.map(p => [p, EclipticLongitude(Body[p as keyof typeof Body], dateObj)])
            )
        );
    }, [date]);

    return <div className="system">
        <div className="sun"></div>
        {Object.keys(planetPositions).map(x =>
            <div key={x} className={`orbit ${x}`} style={{
                transform: `translate(-50%,-50%) rotate(${planetPositions[x as keyof typeof planetPositions]}deg)`
            }}>
                <div className={`planet ${x}`} />
            </div>
        )}
    </div>;
}

export default SolarSystem;
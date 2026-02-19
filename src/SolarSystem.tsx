import { useState, useEffect } from 'react';
import init, { heliocentric_longitudes_at_jde } from '@wasm/natal_chart_solver';
import styles from './SolarSystem.module.css';
import clsx from 'clsx';

function SolarSystem({ date }: { date: Date }) { 
    const [planetPositions, setPlanetPositions] = useState<Float64Array>(new Float64Array([0,0,0,0,0,0,0,0]));

    useEffect(() => {
        init()
            .then(() => {
                const jde = date.getTime() / 86400000.0 + 2440587.5;
                const result = heliocentric_longitudes_at_jde(jde, new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7]));
        
                if (result) {
                setPlanetPositions(result);
                } else { 
                setPlanetPositions(new Float64Array([0,0,0,0,0,0,0,0]));
                }
            })
            .catch((e: unknown) => { 
                setPlanetPositions(new Float64Array([0, 0, 0, 0, 0, 0, 0, 0]))
                console.log(e)
            });
    }, [date]);

    return <div className={styles.system}>
        <div className={styles.sun}></div>
        {
            ['Mercury','Venus','Earth','Mars','Jupiter','Saturn','Uranus','Neptune',].map((p, i) =>
                <div key={p} className={clsx(styles.orbit, styles[p])} style={{transform: `translate(-50%,-50%) rotate(${planetPositions[i]}deg)`}}>
                    <div className={clsx(styles.planet, styles[p])} />
                </div>
            )
        }
    </div>;
}

export default SolarSystem;
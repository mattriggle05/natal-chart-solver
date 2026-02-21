import { useState, useEffect } from 'react';
import init, { system_model_at_date } from '@wasm/natal_chart_solver';
import styles from './SolarSystem.module.css';
import clsx from 'clsx';

function SolarSystem({ date }: { date: Date }) { 
    const [planetPositions, setPlanetPositions] = useState<Float64Array>(new Float64Array([0,0,0,0,0,0,0,0]));

    useEffect(() => {
        init()
            .then(() => {
                const jde = date.getTime() / 86400000.0 + 2440587.5;
                const result = system_model_at_date(jde);
                setPlanetPositions(result);
            })
    }, [date]);

    return <div className={styles.system}> 
        <div className={styles.sun}></div>
        {
            ['mercury','venus','earth','mars','jupiter','saturn','uranus','neptune',].map((p, i) =>
                <div key={p} className={clsx(styles.orbit, styles[p])} style={{transform: `translate(-50%,-50%) rotate(${planetPositions[i]}deg)`}}>
                    <div className={clsx(styles.planet, styles[p])} />
                </div>
            )
        }
    </div>;
}

export default SolarSystem;
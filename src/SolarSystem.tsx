import './SolarSystem.css';

function SolarSystem() { 
    
    const planets = ['mercury', 'venus', 'earth', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune']
    
    return <div className="system">
        <div className="sun"></div>
        {planets.map(x => <div key={x} className={`orbit ${x}`}><div className={`planet ${x}`}></div></div>)}
    </div>;
}

export default SolarSystem;
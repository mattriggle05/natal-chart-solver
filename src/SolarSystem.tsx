import './SolarSystem.css';

function SolarSystem() { 
    
    
    return <div className="system">

        <div className="sun"></div>

        <div className="orbit mercury">
            <div className="planet mercury"></div>
        </div>

        <div className="orbit venus">
            <div className="planet venus"></div>
        </div>

        <div className="orbit earth">
            <div className="planet earth"></div>
        </div>

        <div className="orbit mars">
            <div className="planet mars"></div>
        </div>

        <div className="orbit jupiter">
            <div className="planet jupiter"></div>
        </div>

        <div className="orbit saturn">
            <div className="planet saturn"></div>
        </div>

        <div className="orbit uranus">
            <div className="planet uranus"></div>
        </div>

        <div className="orbit neptune">
            <div className="planet neptune"></div>
        </div>
    </div>;
}

export default SolarSystem;
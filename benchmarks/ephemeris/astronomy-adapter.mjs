import * as A from './generated/astronomy.js';
const names = {0:'Mercury',1:'Venus',3:'Mars',4:'Jupiter',5:'Saturn',6:'Uranus',7:'Neptune',10:'Sun'};
const bodies = [0,1,3,4,5,6,7,10,11,16];
const normalize = x => (x % 360 + 360) % 360;
export function angle(jd, body) {
    if (body === 16) return normalize(angle(jd, 11) - angle(jd, 10));
    const time = A.AstroTime.FromTerrestrialTime(jd - 2451545);
    let v;
    if (body === 11) v = A.GeoMoon(time);
    else {
        const earth = A.HelioVector(A.Body.Earth, time);
        const p = A.HelioVector(names[body], time);
        v = new A.Vector(p.x-earth.x, p.y-earth.y, p.z-earth.z, time);
    }
    return normalize(A.Ecliptic(v).elon - A.e_tilt(time).dpsi / 3600);
}
export function run(body, count, repeats) {
    let checksum = 0;
    for(let repeat=0;repeat<repeats;++repeat)
    for (let i = 0; i < count; ++i) checksum += angle(2415021 + (i * 104729 % 1000003) / 1000002 * 73049, body < 0 ? bodies[i % 10] : body);
    return checksum;
}

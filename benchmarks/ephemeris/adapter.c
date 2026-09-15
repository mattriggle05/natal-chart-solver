#include "astronomy.c"
#include <emscripten/emscripten.h>
static const int bodies[] = {0,1,3,4,5,6,7,10,11,16};
static double normalize(double x) { return fmod(fmod(x, 360.0) + 360.0, 360.0); }
EMSCRIPTEN_KEEPALIVE double angle(double jd, int body) {
    if (body == 16) return normalize(angle(jd, 11) - angle(jd, 10));
    astro_time_t time = Astronomy_TerrestrialTime(jd - 2451545.0);
    astro_vector_t v;
    if (body == 11) v = Astronomy_GeoMoon(time);
    else {
        astro_vector_t earth = Astronomy_HelioVector(BODY_EARTH, time);
        astro_body_t target = body == 10 ? BODY_SUN : (astro_body_t)body;
        v = Astronomy_HelioVector(target, time);
        v.x -= earth.x; v.y -= earth.y; v.z -= earth.z;
    }
    astro_ecliptic_t e = Astronomy_Ecliptic(v);
    return normalize(e.elon - e_tilt(&time).dpsi / 3600.0);
}
EMSCRIPTEN_KEEPALIVE double run(int body, int count, int repeats) {
    double checksum = 0;
    for (int repeat = 0; repeat < repeats; ++repeat) {
        for (int i = 0; i < count; ++i) {
            double jd = 2415021.0 + ((long long)i * 104729 % 1000003) / 1000002.0 * 73049.0;
            checksum += angle(jd, body < 0 ? bodies[i % 10] : body);
        }
    }
    return checksum;
}
#include "search.c"

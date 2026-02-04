"""
Calculate Jupiter's position in the sky (RA and Dec) on January 1, 2026
Using orbital mechanics calculations without astropy or ephemeris libraries.
"""

import math
from datetime import datetime, timedelta

def julian_date(year, month, day, hour=0, minute=0, second=0):
    """Calculate Julian Date from calendar date."""
    if month <= 2:
        year -= 1
        month += 12
    
    A = math.floor(year / 100)
    B = 2 - A + math.floor(A / 4)
    
    JD = math.floor(365.25 * (year + 4716)) + math.floor(30.6001 * (month + 1)) + day + B - 1524.5
    JD += (hour + minute/60 + second/3600) / 24
    
    return JD

def orbital_elements_jupiter(T):
    """
    Calculate Jupiter's orbital elements for a given time.
    T is centuries from J2000.0
    Using VSOP87 simplified elements.
    """
    # Mean longitude (degrees)
    L = 34.351484 + 3034.9056746 * T
    
    # Semi-major axis (AU)
    a = 5.202603191 + 0.0000001913 * T
    
    # Eccentricity
    e = 0.04849485 + 0.000163244 * T
    
    # Inclination (degrees)
    i = 1.30530 - 0.0054966 * T
    
    # Longitude of ascending node (degrees)
    Omega = 100.55615 + 0.1217406 * T
    
    # Longitude of perihelion (degrees)
    pi_long = 14.75385 + 0.2155209 * T
    
    # Mean anomaly (degrees)
    M = L - pi_long
    
    return {
        'a': a,
        'e': e,
        'i': math.radians(i),
        'Omega': math.radians(Omega % 360),
        'omega': math.radians((pi_long - Omega) % 360),
        'M': math.radians(M % 360)
    }

def solve_kepler(M, e, tolerance=1e-8):
    """
    Solve Kepler's equation M = E - e*sin(E) for eccentric anomaly E.
    Using Newton-Raphson iteration.
    """
    E = M
    for _ in range(100):
        delta = E - e * math.sin(E) - M
        if abs(delta) < tolerance:
            break
        E = E - delta / (1 - e * math.cos(E))
    return E

def true_anomaly(E, e):
    """Calculate true anomaly from eccentric anomaly."""
    nu = 2 * math.atan2(
        math.sqrt(1 + e) * math.sin(E / 2),
        math.sqrt(1 - e) * math.cos(E / 2)
    )
    return nu

def heliocentric_position(elements):
    """
    Calculate heliocentric position in ecliptic coordinates.
    Returns (x, y, z) in AU.
    """
    a = elements['a']
    e = elements['e']
    i = elements['i']
    Omega = elements['Omega']
    omega = elements['omega']
    M = elements['M']
    
    # Solve Kepler's equation
    E = solve_kepler(M, e)
    
    # True anomaly
    nu = true_anomaly(E, e)
    
    # Distance from sun
    r = a * (1 - e * math.cos(E))
    
    # Position in orbital plane
    x_orb = r * math.cos(nu)
    y_orb = r * math.sin(nu)
    
    # Rotate to ecliptic coordinates
    x_ecl = (math.cos(omega) * math.cos(Omega) - math.sin(omega) * math.sin(Omega) * math.cos(i)) * x_orb + \
            (-math.sin(omega) * math.cos(Omega) - math.cos(omega) * math.sin(Omega) * math.cos(i)) * y_orb
    
    y_ecl = (math.cos(omega) * math.sin(Omega) + math.sin(omega) * math.cos(Omega) * math.cos(i)) * x_orb + \
            (-math.sin(omega) * math.sin(Omega) + math.cos(omega) * math.cos(Omega) * math.cos(i)) * y_orb
    
    z_ecl = (math.sin(omega) * math.sin(i)) * x_orb + \
            (math.cos(omega) * math.sin(i)) * y_orb
    
    return x_ecl, y_ecl, z_ecl

def earth_position(T):
    """
    Calculate Earth's heliocentric position (simplified).
    T is centuries from J2000.0
    """
    # Earth's mean longitude
    L = 280.460 + 36000.771 * T
    
    # Mean anomaly
    M = math.radians((357.528 + 35999.050 * T) % 360)
    
    # Ecliptic longitude
    lambda_sun = math.radians(L % 360) + math.radians(1.915) * math.sin(M) + \
                 math.radians(0.020) * math.sin(2 * M)
    
    # Distance (AU)
    r = 1.00014 - 0.01671 * math.cos(M) - 0.00014 * math.cos(2 * M)
    
    # Earth's position (heliocentric)
    x = -r * math.cos(lambda_sun)
    y = -r * math.sin(lambda_sun)
    z = 0.0
    
    return x, y, z

def ecliptic_to_equatorial(x, y, z, epsilon):
    """
    Convert ecliptic coordinates to equatorial coordinates.
    epsilon is the obliquity of the ecliptic.
    """
    x_eq = x
    y_eq = y * math.cos(epsilon) - z * math.sin(epsilon)
    z_eq = y * math.sin(epsilon) + z * math.cos(epsilon)
    
    return x_eq, y_eq, z_eq

def rectangular_to_spherical(x, y, z):
    """
    Convert rectangular coordinates to spherical (RA, Dec).
    Returns RA in hours, Dec in degrees.
    """
    r = math.sqrt(x**2 + y**2 + z**2)
    
    # Right Ascension (0 to 24 hours)
    ra_rad = math.atan2(y, x)
    ra_hours = (ra_rad * 12 / math.pi) % 24
    
    # Declination (-90 to +90 degrees)
    dec_rad = math.asin(z / r)
    dec_deg = math.degrees(dec_rad)
    
    return ra_hours, dec_deg, r

def ecliptic_longitude(x, y):
    """
    Calculate ecliptic longitude from geocentric ecliptic coordinates.
    Returns longitude in degrees (0-360).
    """
    lon_rad = math.atan2(y, x)
    lon_deg = math.degrees(lon_rad) % 360
    return lon_deg

def zodiac_sign(longitude):
    """
    Determine zodiac sign from ecliptic longitude.
    Returns sign name and position within sign.
    """
    signs = [
        "Aries", "Taurus", "Gemini", "Cancer", 
        "Leo", "Virgo", "Libra", "Scorpio",
        "Sagittarius", "Capricorn", "Aquarius", "Pisces"
    ]
    
    sign_index = int(longitude / 30)
    position_in_sign = longitude % 30
    
    return signs[sign_index], position_in_sign

def get_data_by_date_jupiter(year, month, day):
    # Target date
    # print(f"Jupiter's position for {month:02d}/{day:02d}/{year}")
    # print("=" * 60)
    
    # Calculate Julian Date
    JD = julian_date(year, month, day)
    # print(f"Julian Date: {JD:.2f}")
    
    # Time in Julian centuries from J2000.0 (JD 2451545.0)
    T = (JD - 2451545.0) / 36525.0
    # print(f"Centuries from J2000.0: {T:.6f}")
    
    # Obliquity of the ecliptic (J2000.0)
    epsilon = math.radians(23.43928)
    # Get Jupiter's orbital elements
    jupiter_elements = orbital_elements_jupiter(T)
    # Calculate heliocentric positions
    jupiter_helio = heliocentric_position(jupiter_elements)
    earth_helio = earth_position(T)
    
    # Calculate geocentric position
    x_geo = jupiter_helio[0] - earth_helio[0]
    y_geo = jupiter_helio[1] - earth_helio[1]
    z_geo = jupiter_helio[2] - earth_helio[2]
    
    # Calculate ecliptic longitude (for zodiac position)
    ecl_longitude = ecliptic_longitude(x_geo, y_geo)
    zodiac, position = zodiac_sign(ecl_longitude)
    
    # print(f"Zodiac Position: {zodiac} {position:.2f}°")
    
    # Convert to equatorial coordinates
    x_eq, y_eq, z_eq = ecliptic_to_equatorial(x_geo, y_geo, z_geo, epsilon)
    
    # print(f"Jupiter geocentric (equatorial): ({x_eq:.6f}, {y_eq:.6f}, {z_eq:.6f}) AU")
    
    # Convert to RA and Dec
    ra_hours, dec_deg, distance = rectangular_to_spherical(x_eq, y_eq, z_eq)
    
    # Convert RA to hours, minutes, seconds
    ra_h = int(ra_hours)
    ra_m = int((ra_hours - ra_h) * 60)
    ra_s = ((ra_hours - ra_h) * 60 - ra_m) * 60
    
    # Convert Dec to degrees, arcminutes, arcseconds
    dec_sign = '+' if dec_deg >= 0 else '-'
    dec_deg_abs = abs(dec_deg)
    dec_d = int(dec_deg_abs)
    dec_m = int((dec_deg_abs - dec_d) * 60)
    dec_s = ((dec_deg_abs - dec_d) * 60 - dec_m) * 60
    
    # print(f"Right Ascension (RA): {ra_h:02d}h {ra_m:02d}m {ra_s:05.2f}s")
    # print(f"                      {ra_hours:.6f} hours")
    # print(f"Declination (Dec):    {dec_sign}{dec_d:02d}° {dec_m:02d}' {dec_s:05.2f}\"")
    # print(f"                      {dec_deg:.6f}°")
    # print(f"Distance from Earth:  {distance:.6f} AU ({distance * 149597870.7:.0f} km)")
    # print()
    # print(f"Ecliptic Longitude:   {ecl_longitude:.6f}°")
    # print(f"Zodiac Sign:          {zodiac} {position:.2f}°")
    
    return zodiac, ecl_longitude

def main():

    start_date = datetime(day=1,month=1,year=2026)
    prev_zodiac = ""
    prev_zodiac_count = 0
    output = ""

    #60190

    for i in range(60190):
        (zodiac, degrees) = get_data_by_date_jupiter(start_date.year,start_date.month,start_date.day)
        if (zodiac == prev_zodiac):
            prev_zodiac_count += 1
        else:
            output += prev_zodiac + " : " + str(prev_zodiac_count) + "\n"
            prev_zodiac = zodiac
            prev_zodiac_count = 1
        print(degrees)
        start_date = start_date + timedelta(days=1)

    print(output)


if __name__ == "__main__":
    main()
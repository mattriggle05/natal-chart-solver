export enum Feature {
    // Planetary longitudes from VSOP87
    Mercury = 0,
    Venus = 1,
    Earth = 2,
    Mars = 3,
    Jupiter = 4,
    Saturn = 5,
    Uranus = 6,
    Neptune = 7,

    // Other searchable body longitudes
    Sun = 10,
    Moon = 11,

    // Unsupported objects
    Pluto = 12,
    Chiron = 15,

    // Unsupported calculated points
    NorthNode = 13,
    Lilith = 14,

    // Derived searchable features
    MoonPhaseAngle = 16,
}

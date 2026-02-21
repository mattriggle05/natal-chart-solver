export enum Feature {
    // Planets (all in vsop87)
    Mercury = 0,
    Venus = 1,
    Earth = 2,
    Mars = 3,
    Jupiter = 4,
    Saturn = 5,
    Uranus = 6,
    Neptune = 7,

    // Solar system bodies in vsop87
    Sun = 10,
    Moon = 11,

    // objects not in vsop87
    Pluto = 12,
    Chiron = 15,

    // calculated points
    NorthNode = 13,
    Lilith = 14,
}
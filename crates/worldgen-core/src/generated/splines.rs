/* Generated from pinned Pumpkin. Do not edit manually. */
pub struct SplinePoint {
    pub location: f32,
    pub value: &'static SplineRepr,
    pub derivative: f32,
}

pub enum SplineRepr {
    Standard {
        location_function_index: usize,
        points: &'static [SplinePoint],
    },
    Fixed {
        value: f32,
    },
}

pub static SPLINE_27: SplineRepr = SplineRepr::Standard {
    location_function_index: 17usize,
    points: &[
        SplinePoint {
            location: -1.1f32,
            value: &SplineRepr::Fixed { value: 0.044f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: -1.02f32,
            value: &SplineRepr::Fixed { value: -0.2222f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.51f32,
            value: &SplineRepr::Fixed { value: -0.2222f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.44f32,
            value: &SplineRepr::Fixed { value: -0.12f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.18f32,
            value: &SplineRepr::Fixed { value: -0.12f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.16f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.85f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: -0.08880186f32,
                                    },
                                    derivative: 0.38940096f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.69000006f32,
                                    },
                                    derivative: 0.38940096f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: -0.115760356f32,
                                    },
                                    derivative: 0.37788022f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.6400001f32,
                                    },
                                    derivative: 0.37788022f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.75f32,
                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.65f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.5954547f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.000000029802322f32,
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.6054547f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.000000029802322f32,
                                    },
                                    derivative: 0.2534563f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.100000024f32,
                                    },
                                    derivative: 0.2534563f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.060000002f32,
                                    },
                                    derivative: 0.007000001f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0.1f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.060000002f32,
                                    },
                                    derivative: 0.007000001f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.2f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0.06f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.15f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.85f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: -0.08880186f32,
                                    },
                                    derivative: 0.38940096f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.69000006f32,
                                    },
                                    derivative: 0.38940096f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: -0.115760356f32,
                                    },
                                    derivative: 0.37788022f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.6400001f32,
                                    },
                                    derivative: 0.37788022f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.75f32,
                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.65f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.5954547f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.000000029802322f32,
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.6054547f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.000000029802322f32,
                                    },
                                    derivative: 0.2534563f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.100000024f32,
                                    },
                                    derivative: 0.2534563f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.3f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.060000002f32,
                                    },
                                    derivative: 0.007000001f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0.1f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.060000002f32,
                                    },
                                    derivative: 0.007000001f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.2f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.15f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0.06f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.1f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.85f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: -0.08880186f32,
                                    },
                                    derivative: 0.38940096f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.69000006f32,
                                    },
                                    derivative: 0.38940096f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: -0.115760356f32,
                                    },
                                    derivative: 0.37788022f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.6400001f32,
                                    },
                                    derivative: 0.37788022f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.75f32,
                                    value: &SplineRepr::Fixed { value: -0.2222f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.65f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.5954547f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.000000029802322f32,
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.6054547f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.000000029802322f32,
                                    },
                                    derivative: 0.2534563f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.100000024f32,
                                    },
                                    derivative: 0.2534563f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.060000002f32,
                                    },
                                    derivative: 0.007000001f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.001f32 },
                                    derivative: 0.01f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.003f32 },
                                    derivative: 0.01f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.05f32 },
                                    derivative: 0.094000004f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.060000002f32,
                                    },
                                    derivative: 0.007000001f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.2f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.12f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: 0.25f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.85f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.20235021f32,
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.7161751f32,
                                    },
                                    derivative: 0.5138249f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 1.23f32 },
                                    derivative: 0.5138249f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.44682026f32,
                                    },
                                    derivative: 0.43317974f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.88f32 },
                                    derivative: 0.43317974f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.30829495f32,
                                    },
                                    derivative: 0.3917051f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.70000005f32,
                                    },
                                    derivative: 0.3917051f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.25f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.42000002f32,
                                    },
                                    derivative: 0.049000014f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.0069999998f32,
                                    },
                                    derivative: 0.07f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.021f32 },
                                    derivative: 0.07f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.35f32 },
                                    derivative: 0.658f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.42000002f32,
                                    },
                                    derivative: 0.049000014f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.2f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.45f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 26usize,
                                        points: &[
                                            SplinePoint {
                                                location: -1f32,
                                                value: &SplineRepr::Fixed { value: -0.1f32 },
                                                derivative: 0.5f32,
                                            },
                                            SplinePoint {
                                                location: -0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.03f32 },
                                                derivative: 0.04f32,
                                            },
                                            SplinePoint {
                                                location: 1f32,
                                                value: &SplineRepr::Fixed { value: 0.1f32 },
                                                derivative: 0.049f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.55f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 26usize,
                                        points: &[
                                            SplinePoint {
                                                location: -1f32,
                                                value: &SplineRepr::Fixed { value: -0.1f32 },
                                                derivative: 0.5f32,
                                            },
                                            SplinePoint {
                                                location: -0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.03f32 },
                                                derivative: 0.04f32,
                                            },
                                            SplinePoint {
                                                location: 1f32,
                                                value: &SplineRepr::Fixed { value: 0.1f32 },
                                                derivative: 0.049f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.58f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.1f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: -0.03f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.12f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: 1f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.85f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.34792626f32,
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.9239631f32,
                                    },
                                    derivative: 0.5760369f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 1.5f32 },
                                    derivative: 0.5760369f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.5391705f32,
                                    },
                                    derivative: 0.4608295f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 1f32 },
                                    derivative: 0.4608295f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: 0.2f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed {
                                        value: 0.5391705f32,
                                    },
                                    derivative: 0.4608295f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 1f32 },
                                    derivative: 0.4608295f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.2f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                    derivative: 0.070000015f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0.099999994f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.099999994f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.5f32 },
                                    derivative: 0.94f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.6f32 },
                                    derivative: 0.070000015f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.2f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.45f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 26usize,
                                        points: &[
                                            SplinePoint {
                                                location: -1f32,
                                                value: &SplineRepr::Fixed { value: -0.05f32 },
                                                derivative: 0.5f32,
                                            },
                                            SplinePoint {
                                                location: -0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.03f32 },
                                                derivative: 0.04f32,
                                            },
                                            SplinePoint {
                                                location: 1f32,
                                                value: &SplineRepr::Fixed { value: 0.1f32 },
                                                derivative: 0.049f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.55f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 26usize,
                                        points: &[
                                            SplinePoint {
                                                location: -1f32,
                                                value: &SplineRepr::Fixed { value: -0.05f32 },
                                                derivative: 0.5f32,
                                            },
                                            SplinePoint {
                                                location: -0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 0.01f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.4f32,
                                                value: &SplineRepr::Fixed { value: 0.03f32 },
                                                derivative: 0.04f32,
                                            },
                                            SplinePoint {
                                                location: 1f32,
                                                value: &SplineRepr::Fixed { value: 0.1f32 },
                                                derivative: 0.049f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.17f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.58f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.05f32 },
                                    derivative: 0.5f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.7f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -1f32,
                                    value: &SplineRepr::Fixed { value: -0.02f32 },
                                    derivative: 0.015f32,
                                },
                                SplinePoint {
                                    location: -0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0f32,
                                    value: &SplineRepr::Fixed { value: 0.01f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.4f32,
                                    value: &SplineRepr::Fixed { value: 0.03f32 },
                                    derivative: 0.04f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Fixed { value: 0.1f32 },
                                    derivative: 0.049f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
    ],
};

pub static SPLINE_34: SplineRepr = SplineRepr::Standard {
    location_function_index: 17usize,
    points: &[
        SplinePoint {
            location: -0.11f32,
            value: &SplineRepr::Fixed { value: 0f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: 0.03f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.19999999f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.44999996f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.63f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.3f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.78f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.19999999f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.44999996f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.315f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.15f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.5775f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.19999999f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.44999996f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.315f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.15f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.375f32,
                        value: &SplineRepr::Fixed { value: 0f32 },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: 0.65f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.19999999f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.44999996f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.63f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.3f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.63f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.3f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.78f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.19999999f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.44999996f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.63f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.3f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.5775f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.19999999f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.44999996f32,
                                    value: &SplineRepr::Fixed { value: 0f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 1f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.63f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.01f32,
                                                value: &SplineRepr::Fixed { value: 0.3f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.375f32,
                        value: &SplineRepr::Fixed { value: 0f32 },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
    ],
};

pub static SPLINE_43: SplineRepr = SplineRepr::Standard {
    location_function_index: 17usize,
    points: &[
        SplinePoint {
            location: -0.19f32,
            value: &SplineRepr::Fixed { value: 3.95f32 },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.15f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.6f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.5f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.25f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.03f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.35f32,
                        value: &SplineRepr::Fixed { value: 6.25f32 },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.45f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.9f32,
                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.69f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 6.25f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.1f32,
                                                value: &SplineRepr::Fixed { value: 0.625f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.55f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.9f32,
                                    value: &SplineRepr::Fixed { value: 6.25f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.69f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 6.25f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.1f32,
                                                value: &SplineRepr::Fixed { value: 0.625f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.62f32,
                        value: &SplineRepr::Fixed { value: 6.25f32 },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: -0.1f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.6f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.5f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.25f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.03f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.35f32,
                        value: &SplineRepr::Fixed { value: 5.47f32 },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.45f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.9f32,
                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.69f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 5.47f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.1f32,
                                                value: &SplineRepr::Fixed { value: 0.625f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.55f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.9f32,
                                    value: &SplineRepr::Fixed { value: 5.47f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.69f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 5.47f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.1f32,
                                                value: &SplineRepr::Fixed { value: 0.625f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.62f32,
                        value: &SplineRepr::Fixed { value: 5.47f32 },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: 0.03f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.6f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.5f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.25f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.03f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.35f32,
                        value: &SplineRepr::Fixed { value: 5.08f32 },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.45f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.9f32,
                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.69f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 5.08f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.1f32,
                                                value: &SplineRepr::Fixed { value: 0.625f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.55f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.9f32,
                                    value: &SplineRepr::Fixed { value: 5.08f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.69f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: 0f32,
                                                value: &SplineRepr::Fixed { value: 5.08f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.1f32,
                                                value: &SplineRepr::Fixed { value: 0.625f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.62f32,
                        value: &SplineRepr::Fixed { value: 5.08f32 },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
        SplinePoint {
            location: 0.06f32,
            value: &SplineRepr::Standard {
                location_function_index: 19usize,
                points: &[
                    SplinePoint {
                        location: -0.6f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.5f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.35f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.25f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: -0.1f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.05f32,
                                    value: &SplineRepr::Fixed { value: 2.67f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.05f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.03f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 21usize,
                            points: &[
                                SplinePoint {
                                    location: -0.2f32,
                                    value: &SplineRepr::Fixed { value: 6.3f32 },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.2f32,
                                    value: &SplineRepr::Fixed { value: 4.69f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.05f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.45f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.2f32,
                                                value: &SplineRepr::Fixed { value: 6.3f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.2f32,
                                                value: &SplineRepr::Fixed { value: 4.69f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.7f32,
                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.4f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: 0.45f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.2f32,
                                                value: &SplineRepr::Fixed { value: 6.3f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.2f32,
                                                value: &SplineRepr::Fixed { value: 4.69f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: 0.7f32,
                                    value: &SplineRepr::Fixed { value: 1.56f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.45f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.7f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.2f32,
                                                value: &SplineRepr::Fixed { value: 6.3f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.2f32,
                                                value: &SplineRepr::Fixed { value: 4.69f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.15f32,
                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.55f32,
                        value: &SplineRepr::Standard {
                            location_function_index: 26usize,
                            points: &[
                                SplinePoint {
                                    location: -0.7f32,
                                    value: &SplineRepr::Standard {
                                        location_function_index: 21usize,
                                        points: &[
                                            SplinePoint {
                                                location: -0.2f32,
                                                value: &SplineRepr::Fixed { value: 6.3f32 },
                                                derivative: 0f32,
                                            },
                                            SplinePoint {
                                                location: 0.2f32,
                                                value: &SplineRepr::Fixed { value: 4.69f32 },
                                                derivative: 0f32,
                                            },
                                        ],
                                    },
                                    derivative: 0f32,
                                },
                                SplinePoint {
                                    location: -0.15f32,
                                    value: &SplineRepr::Fixed { value: 1.37f32 },
                                    derivative: 0f32,
                                },
                            ],
                        },
                        derivative: 0f32,
                    },
                    SplinePoint {
                        location: 0.58f32,
                        value: &SplineRepr::Fixed { value: 4.69f32 },
                        derivative: 0f32,
                    },
                ],
            },
            derivative: 0f32,
        },
    ],
};

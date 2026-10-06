import type { Feature } from './features';

export interface SearchParams {
    startDate: string;
    endDate: string;
    featureIds: Feature[];
    angleStarts: number[];
    angleSpans: number[];
}

export type SearchResponse =
    | { type: 'searching' }
    | { type: 'complete'; result: Float64Array }
    | { type: 'error'; message: string };

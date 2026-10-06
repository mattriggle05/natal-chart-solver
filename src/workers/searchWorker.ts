import init, { search, search_date_range } from '@wasm/natal_chart_solver';
import { civilDateToJdTt } from '../utils/time';
import type { SearchParams, SearchResponse } from '../types/search';

function respond(message: SearchResponse) { self.postMessage(message); }

self.onmessage = async (event: MessageEvent<SearchParams>) => {
    try {
        await init();
        const { startDate, endDate, featureIds, angleStarts, angleSpans } = event.data;
        let start = civilDateToJdTt(startDate);
        let end = civilDateToJdTt(endDate);
        if (startDate < '1900-01-01' || endDate > '2100-01-01') throw new RangeError('Choose dates between 1900 and 2100.');
        const bounds = search_date_range();
        // The edge date controls represent the supported domain endpoints.
        if (startDate === '1900-01-01') start = bounds[0] as typeof start;
        if (endDate === '2100-01-01') end = bounds[1] as typeof end;
        respond({ type: 'searching' });
        const result = search(start, end, new Uint8Array(featureIds), new Float64Array(angleStarts), new Float64Array(angleSpans));
        respond({ type: 'complete', result });
    } catch (error) {
        respond({ type: 'error', message: String(error) });
    }
};

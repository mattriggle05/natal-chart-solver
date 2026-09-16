import init, { search } from '@wasm/natal_chart_solver';

const ready = init();

self.onmessage = async (e) => {
    try {
        await ready;

        if (e.data.type === 'search') {
            console.log('worker searching')

            const { startJdTt, endJdTt, featureIds, angleStarts, angleSpans } = e.data.params;

            console.log({ startJdTt, endJdTt, featureIds, angleStarts, angleSpans })

            let result = search(
                startJdTt,
                endJdTt,
                new Uint8Array(featureIds),
                new Float64Array(angleStarts),
                new Float64Array(angleSpans),
            );

            console.log('worker finished')
            self.postMessage({ type: 'complete', result });
        }
    } catch (err) {
        self.postMessage({ type: 'ERROR', message: String(err) });
    }
};

import init, { search } from '@wasm/natal_chart_solver';

const ready = init();

self.onmessage = async (e) => {
    try {
        await ready;

        if (e.data.type === 'search') {
            console.log('worker searching')

            const { startJd, endJd, featureIds, featureSigns } = e.data.params;

            console.log({ startJd, endJd, featureIds, featureSigns })

            let result = search(
                startJd,
                endJd,
                new Uint8Array(featureIds),
                new Uint8Array(featureSigns),
            );

            console.log('worker finished')
            self.postMessage({ type: 'complete', result });
        }
    } catch (err) {
        self.postMessage({ type: 'ERROR', message: String(err) });
    }
};

import { useState, useEffect, useRef, useCallback } from 'react';

export interface SearchParams {
    startJd: number;
    endJd: number;
    featureIds: number[];
    featureSigns: number[];
}

export function useDataSearch() {
    const workerRef = useRef<Worker | null>(null);
    const [results, setResults] = useState<Float64Array>(new Float64Array([-1]));

    useEffect(() => {
        workerRef.current = new Worker(
            new URL('../workers/searchWorker.ts', import.meta.url),
            { type: 'module' }
        );

        workerRef.current.onmessage = (e) => {
            if (e.data.type === 'complete') {
                console.log('complete received')
                const result = e.data.result as Float64Array;
                console.log(e.data)
                console.log(result)
                setResults(result);
            }
        };

        return () => workerRef.current?.terminate();
    }, []);

    const search = useCallback((params: SearchParams) => {
        console.log('search called')
        console.log(params)
        setResults(new Float64Array());
        workerRef.current?.postMessage({ type: 'search', params });
    }, []);

    return { search, results }
}
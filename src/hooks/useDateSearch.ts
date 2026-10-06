import { useState, useEffect, useRef } from 'react';
import { formatWindowMinutes } from '../utils/time';
import type { JulianDateTt } from '../utils/time';
import type { SearchParams, SearchResponse } from '../types/search';

export function useDataSearch() {
    const workerRef = useRef<Worker | null>(null);
    const [busy, setBusy] = useState(false);
    const [output, setOutput] = useState('Choose signs and dates, then select Search.');
    useEffect(() => () => workerRef.current?.terminate(), []);

    function cancel() {
        workerRef.current?.terminate();
        workerRef.current = null;
        setBusy(false);
        setOutput('Search cancelled.');
    }

    function search(params: SearchParams) {
        workerRef.current?.terminate();
        setBusy(true);
        setOutput('Initializing search…');
        let worker: Worker;
        try {
            worker = new Worker(new URL('../workers/searchWorker.ts', import.meta.url), { type: 'module' });
            workerRef.current = worker;
            const finish = (message: string) => {
                if (workerRef.current !== worker) return;
                setOutput(message);
                setBusy(false);
                worker.terminate();
                workerRef.current = null;
            };
            worker.onerror = event => finish(`Error: ${event.message || 'Search worker failed.'}`);
            worker.onmessageerror = () => finish('Error: Could not read the worker response.');
            worker.onmessage = (event: MessageEvent<SearchResponse>) => {
                if (workerRef.current !== worker) return;
                const message = event.data;
                if (message.type === 'searching') { setOutput('Searching…'); return; }
                if (message.type === 'error') { finish(`Error: ${message.message}`); return; }
                const windows: string[] = [];
                for (let i = 0; i < message.result.length; i += 2) {
                    windows.push(formatWindowMinutes(message.result[i] as JulianDateTt, message.result[i + 1] as JulianDateTt));
                }
                finish(windows.length ? `${windows.length} matching interval(s)\n${windows.join('\n')}` : 'No matching intervals.');
            };
            worker.postMessage(params);
        } catch (error) {
            workerRef.current?.terminate();
            workerRef.current = null;
            setBusy(false);
            setOutput(`Error: ${String(error)}`);
        }
    }
    return { search, cancel, busy, output };
}

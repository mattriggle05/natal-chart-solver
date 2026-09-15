const started = performance.now();
const engine = new URL(import.meta.url).searchParams.get('engine');
let run, sample, search;
try {
    if (engine === 'javascript') {
        const module = await import('./astronomy-adapter.mjs');
        run = module.run;
        const query = await import('./search.mjs');
        search = () => query.search(module.angle);
        sample = (dates, body) => dates.map(jd => module.angle(jd, body));
    } else if (engine === 'c') {
        const module = await import('./generated/astronomy-c.mjs');
        const wasm = await module.default();
        run = wasm._run;
        search = () => { const n=wasm._representative_search(); if(n<0)throw Error('Search failed: '+n); return Array.from({length:n},(_,i)=>wasm._search_value(i)); };
        sample = (dates, body) => dates.map(jd => wasm._angle(jd, body));
    } else {
        const module = await import(`./generated/${engine}/ephemeris_lab.js`);
        await module.default();
        run = module.run;
        sample = module.sample;
        search = module.representative_search;
    }
    postMessage({type:'ready', moduleMs:performance.now()-started});
    onmessage = ({data}) => {
        try {
            if (data.type === 'search') { const start=performance.now(); const windows=search(); postMessage({elapsedMs:performance.now()-start,windows:Array.from(windows)}); }
            else if (data.type === 'sample') postMessage({values:Array.from(sample(data.dates, data.body))});
            else {
                const start = performance.now();
                const checksum = run(data.body, data.count, data.repeats ?? 1);
                const elapsedMs = performance.now()-start;
                if (!Number.isFinite(checksum)) throw Error('Nonfinite checksum');
                postMessage({elapsedMs, checksum});
            }
        } catch (error) { postMessage({error:String(error)}); }
    };
} catch (error) { postMessage({error:String(error)}); }

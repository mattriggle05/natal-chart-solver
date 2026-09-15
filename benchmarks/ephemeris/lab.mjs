const engines = ['baseline','astro','javascript','c'];
const bodies = [0,1,3,4,5,6,7,10,11,16,-1];
const names = ['Mercury','Venus','Mars','Jupiter','Saturn','Uranus','Neptune','Sun','Moon','Moon phase','Mixed'];
const output = document.querySelector('#output');
const setBusy = busy => { document.querySelector('#run').disabled=busy; document.querySelector('#finalists').disabled=busy; };
async function record(name,result) { const response=await fetch('/record/'+name,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(result)}); if(!response.ok)throw Error('Could not save results: '+response.status); }
const log = text => { output.textContent += '\n' + text; };
const difference = (a,b) => ((a-b+540)%360)-180;
function receive(worker) {
    return new Promise((resolve,reject) => {
        const timer = setTimeout(() => reject(Error('Worker timeout')), 120000);
        worker.onmessage = ({data}) => { clearTimeout(timer); data.error ? reject(Error(data.error)) : resolve(data); };
        worker.onerror = event => { clearTimeout(timer); reject(Error(event.message)); };
    });
}
async function request(worker, message) { const pending = receive(worker); worker.postMessage(message); return pending; }
let seed = 20260913;
function shuffle(values) {
    const copy = [...values];
    for (let i=copy.length-1;i>0;--i) { seed = (Math.imul(seed,1664525)+1013904223)>>>0; const j=seed%(i+1); [copy[i],copy[j]]=[copy[j],copy[i]]; }
    return copy;
}
function stats(values) { const s=[...values].sort((a,b)=>a-b); return {median:s[Math.floor(s.length/2)],p95:s[Math.ceil(s.length*.95)-1]}; }
const fixtures = {10:[280.6632438,280.3681519,281.1128353],11:[279.6166444,223.3148557,164.4125136],16:[358.9534006,302.9467038,243.2996783]};
export async function benchmark() {
    seed = 20260913;
    const workers = {};
    const result = {timestamp:new Date().toISOString(),userAgent:navigator.userAgent,hardwareConcurrency:navigator.hardwareConcurrency,seed:20260913,count:2000,repetitions:9,initialization:{},calibration:[],trials:[],throughput:[],accuracy:[],fixtures:[],visibilityChanges:0};
    const visibility = () => ++result.visibilityChanges;
    document.addEventListener('visibilitychange',visibility);
    output.textContent='Initializing workers';
    try {
        for (const engine of shuffle(engines)) {
            const start=performance.now();
            const worker=new Worker(`./worker.mjs?engine=${engine}`,{type:'module'});
            workers[engine]=worker;
            const ready=await receive(worker);
            result.initialization[engine]={workerReadyMs:performance.now()-start,moduleMs:ready.moduleMs};
        }
        log('Warming all body paths');
        for (const engine of engines) for (const body of bodies) await request(workers[engine],{body,count:1000});
        const batches={};
        for(const engine of engines)for(const body of bodies) {
            const timing=await request(workers[engine],{body,count:result.count});
            const repeats=Math.max(1,Math.min(100,Math.ceil(60/Math.max(timing.elapsedMs,0.1))));
            batches[engine+':'+body]=repeats;
            result.calibration.push({engine,body,repeats,elapsedMs:timing.elapsedMs});
        }
        for (let trial=0;trial<result.repetitions;++trial) {
            for (const body of shuffle(bodies)) for (const engine of shuffle(engines)) {
                const repeats=batches[engine+':'+body];
                const timing=await request(workers[engine],{body,count:result.count,repeats});
                result.trials.push({trial,body,engine,repeats,...timing});
            }
            log(`Completed trial ${trial+1}/${result.repetitions}`);
        }
        for (const body of bodies) for (const engine of engines) {
            const trials=result.trials.filter(t=>t.engine===engine&&t.body===body);
            if (trials.some(t=>t.checksum!==trials[0].checksum)) throw Error('Unstable checksum');
            const timing=stats(trials.map(t=>t.elapsedMs));
            result.throughput.push({engine,body,name:names[bodies.indexOf(body)],medianMs:timing.median,p95Ms:timing.p95,evaluationsPerSecond:result.count*trials[0].repeats*1000/timing.median});
        }
        log('Comparing 1,001 dates including the 1900 and 2100 endpoints');
        const dates=Array.from({length:1001},(_,i)=>2415021+i/1000*73049);
        for (const body of bodies.filter(b=>b>=0)) {
            const baseline=(await request(workers.baseline,{type:'sample',dates,body})).values;
            for (const engine of engines) {
                const values=(await request(workers[engine],{type:'sample',dates,body})).values;
                const errors=values.map((v,i)=>difference(v,baseline[i]));
                if (errors.some(v=>!Number.isFinite(v))) throw Error('Nonfinite accuracy sample');
                result.accuracy.push({engine,body,maxDegrees:Math.max(...errors.map(Math.abs)),rmsDegrees:Math.sqrt(errors.reduce((s,v)=>s+v*v,0)/errors.length)});
                if (fixtures[body]) {
                    const actual=(await request(workers[engine],{type:'sample',dates:[2415021,2451545,2488070],body})).values;
                    result.fixtures.push({engine,body,actual,errorsDegrees:actual.map((v,i)=>difference(v,fixtures[body][i]))});
                }
            }
        }
        await record('throughput',result);
        window.benchmarkResult=result;
        log(JSON.stringify(result.throughput,null,2));
        document.querySelector('#save').disabled=false;
        return result;
    } finally { Object.values(workers).forEach(w=>w.terminate()); document.removeEventListener('visibilitychange',visibility); }
}
window.runBenchmark=benchmark;
document.querySelector('#run').onclick=async()=>{ setBusy(true); try {await benchmark();}catch(e){log(String(e));}finally{setBusy(false);} };
document.querySelector('#save').onclick=()=>{const a=document.createElement('a');a.href=URL.createObjectURL(new Blob([JSON.stringify(window.benchmarkResult,null,2)],{type:'application/json'}));a.download='ephemeris-results.json';a.click();URL.revokeObjectURL(a.href);};

document.querySelector('#finalists').onclick=async()=>{
    seed=20260913;
    const workers={}, result={timestamp:new Date().toISOString(),userAgent:navigator.userAgent,seed:20260913,trials:[],comparison:[]};
    setBusy(true);
    output.textContent='Running complete finalist searches';
    try {
        for(const engine of ['baseline','javascript','c']) {
            const worker=new Worker(`./worker.mjs?engine=${engine}`,{type:'module'});workers[engine]=worker;await receive(worker);
            await request(worker,{type:'search'});
        }
        for(let trial=0;trial<9;++trial)for(const engine of shuffle(['baseline','javascript','c']))result.trials.push({trial,engine,...await request(workers[engine],{type:'search'})});
        const reference=result.trials.find(t=>t.engine==='baseline').windows;
        for(const engine of ['baseline','javascript','c']) {
            const trials=result.trials.filter(t=>t.engine===engine),windows=trials[0].windows;
            if(trials.some(t=>JSON.stringify(t.windows)!==JSON.stringify(windows)))throw Error('Unstable search results');
            result.comparison.push({engine,windowCount:windows.length/2,...stats(trials.map(t=>t.elapsedMs)),sameWindowCount:windows.length===reference.length,maxBoundaryShiftMinutes:windows.length===reference.length?Math.max(...windows.map((v,i)=>Math.abs(v-reference[i])*1440)):null});
        }
        await record('search',result);
        log(JSON.stringify(result.comparison,null,2));
    }catch(e){log(String(e));}finally{Object.values(workers).forEach(w=>w.terminate());setBusy(false);}
};

import {readFileSync} from 'node:fs';
import {gzipSync,brotliCompressSync,constants} from 'node:zlib';
import {createHash} from 'node:crypto';
const groups = {baseline:['generated/baseline/ephemeris_lab.js','generated/baseline/ephemeris_lab_bg.wasm'],astro:['generated/astro/ephemeris_lab.js','generated/astro/ephemeris_lab_bg.wasm'],javascript:['search.mjs','astronomy-adapter.mjs','generated/astronomy.js'],c:['generated/astronomy-c.mjs','generated/astronomy-c.wasm']};
const report={};
for (const [engine,files] of Object.entries(groups)) {
    const assets=files.map(path=>{const bytes=readFileSync(new URL(path,import.meta.url));return {path,raw:bytes.length,gzip:gzipSync(bytes,{level:9}).length,brotli:brotliCompressSync(bytes,{params:{[constants.BROTLI_PARAM_QUALITY]:11}}).length,sha256:createHash('sha256').update(bytes).digest('hex')};});
    report[engine]={assets,totals:Object.fromEntries(['raw','gzip','brotli'].map(k=>[k,assets.reduce((n,a)=>n+a[k],0)]))};
}
console.log(JSON.stringify(report,null,2));

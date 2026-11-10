import {run} from './raw.mjs';
self.onmessage=async()=>{
 try {
  const bytes=await (await fetch(new URL('../../wasm/pkg/natal_chart_solver_bg.wasm',import.meta.url))).arrayBuffer();
  run(bytes,result=>self.postMessage(result));
  self.postMessage({done:true});
 } catch(e){ self.postMessage({error:String(e)}); }
};

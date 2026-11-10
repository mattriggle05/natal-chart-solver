import {initSync, search} from '../../wasm/pkg/natal_chart_solver.js';
export const cases = [
 ['sun200',2415020.5,2488069.5,[10],[0]],
 ['sunmoon200',2415020.5,2488069.5,[10,11],[150,0]],
 ['mercury200',2415020.5,2488069.5,[0],[0]],
];
export function run(bytes, emit) {
 initSync({module:bytes});
 for(const [name,start,end,ids,angles] of cases) {
  for(let trial=1;trial<=2;trial++) {
   const t=performance.now();
   const result=search(start,end,new Uint8Array(ids),new Float64Array(angles),new Float64Array(ids.map(()=>30)));
   emit({name,trial,ms:performance.now()-t,windows:result.length/2,checksum:result.reduce((a,b)=>a+b,0)});
  }
 }
}

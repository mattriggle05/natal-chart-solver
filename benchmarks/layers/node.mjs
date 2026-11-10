import {readFileSync} from 'node:fs';
import {run} from './raw.mjs';
run(readFileSync(new URL('../../wasm/pkg/natal_chart_solver_bg.wasm',import.meta.url)), result=>console.log(JSON.stringify(result)));

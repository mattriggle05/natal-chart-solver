import http from 'node:http';
import {readFile,writeFile} from 'node:fs/promises';
const root=new URL('./',import.meta.url);
http.createServer(async(req,res)=>{
    try {
        const url=new URL(req.url,'http://127.0.0.1:8766');
        if(req.method==='POST'&&['/record/throughput','/record/search'].includes(url.pathname)) {
            if(req.headers.origin!=='http://127.0.0.1:8766') {res.writeHead(403).end();return;}
            let content='';for await(const chunk of req){content+=chunk;if(content.length>2000000){res.writeHead(413).end();return;}}
            const value=JSON.parse(content);
            await writeFile(new URL(`results/${url.pathname.split('/').pop()}.json`,root),JSON.stringify(value,null,2)+'\n');
            res.writeHead(200).end('Saved');return;
        }
        const path=url.pathname==='/'?'index.html':decodeURIComponent(url.pathname.slice(1));
        if(path.includes('..')){res.writeHead(403).end();return;}
        const file=new URL('./'+path,root);
        if(!file.href.startsWith(root.href)){res.writeHead(403).end();return;}
        const bytes=await readFile(file);
        const extension=path.split('.').pop();
        res.writeHead(200,{'Content-Type':({html:'text/html',mjs:'text/javascript',js:'text/javascript',wasm:'application/wasm',json:'application/json'})[extension]||'text/plain','Cache-Control':'no-store'}).end(bytes);
    }catch{res.writeHead(404).end('Not found');}
}).listen(8766,'127.0.0.1',()=>console.log('Ephemeris laboratory: http://127.0.0.1:8766'));

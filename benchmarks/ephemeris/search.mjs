// Isolated monotonic-path search: Sun [150,180), then Moon [0,30), 1900–2100 TT.
// Same 28/10-day steps and one-minute bisection termination as production.
const norm = x => (x % 360 + 360) % 360;
const diff = (a,b) => norm(a-b+180)-180;
export function search(angle) {
    let windows = [[2415021,2488069.5]];
    for (const [body,start,step] of [[10,150,28],[11,0,10]]) {
        const next=[];
        for (const [a,b] of windows) {
            let left=a, value=angle(a,body), open=norm(value-start)<30?a:null;
            while(left<b) {
                const right=Math.min(left+step,b), end=angle(right,body), displacement=diff(end,value);
                if (!(displacement>0&&displacement<180)) throw Error('Monotonic search precondition failed');
                const crossings=[start,start+30].map(target=>({target,distance:norm(target-value)})).sort((a,b)=>a.distance-b.distance);
                for(const {target,distance} of crossings) {
                    if(distance>0&&distance<=displacement) {
                        let lo=left,hi=right;
                        while(hi-lo>=1/1440) {const mid=(lo+hi)/2;if(diff(angle(mid,body),target)<0)lo=mid;else hi=mid;}
                        const root=(lo+hi)/2;
                        if(open===null)open=root;else{if(open<root)next.push([open,root]);open=null;}
                    }
                }
                left=right;value=end;
            }
            if(open!==null&&open<b)next.push([open,b]);
        }
        windows=next;
    }
    return windows.flat();
}

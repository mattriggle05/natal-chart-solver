// Included after adapter.c: identical monotonic query to search.mjs.
static double search_windows[2][20000];
static int search_length;
static double difference(double a, double b) { return normalize(a-b+180)-180; }
EMSCRIPTEN_KEEPALIVE int representative_search(void) {
    int length=2, source=0;
    search_windows[0][0]=2415021;search_windows[0][1]=2488069.5;
    for(int stage=0;stage<2;++stage) {
        int body=stage==0?10:11, count=0;
        double start=stage==0?150:0, step=stage==0?28:10;
        double *input=search_windows[source], *output=search_windows[1-source];
        for(int w=0;w<length;w+=2) {
            double left=input[w], limit=input[w+1], value=angle(left,body);
            double open=normalize(value-start)<30?left:-1;
            while(left<limit) {
                double right=fmin(left+step,limit), end=angle(right,body), displacement=difference(end,value);
                if(!(displacement>0&&displacement<180))return -1;
                double targets[2]={start,start+30}, distances[2]={normalize(start-value),normalize(start+30-value)};
                if(distances[0]>distances[1]) { double t=targets[0];targets[0]=targets[1];targets[1]=t;t=distances[0];distances[0]=distances[1];distances[1]=t; }
                for(int j=0;j<2;++j)if(distances[j]>0&&distances[j]<=displacement) {
                    double lo=left,hi=right;
                    while(hi-lo>=1.0/1440) {double mid=(lo+hi)/2;if(difference(angle(mid,body),targets[j])<0)lo=mid;else hi=mid;}
                    double root=(lo+hi)/2;
                    if(open<0)open=root;else{if(count+2>20000)return -2;if(open<root){output[count++]=open;output[count++]=root;}open=-1;}
                }
                left=right;value=end;
            }
            if(open>=0&&open<limit){if(count+2>20000)return -2;output[count++]=open;output[count++]=limit;}
        }
        source=1-source;length=count;
    }
    search_length=length;return length;
}
EMSCRIPTEN_KEEPALIVE double search_value(int index) {return index>=0&&index<search_length?search_windows[0][index]:NAN;}

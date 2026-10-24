// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// gunzip(base64) -> parsed JSON. Uses the browser's DecompressionStream when there is one, and a small inflate (after puff.c) when not.
// Injected between the gunzip markers of the site template and of the studio tour by the build scripts; keep it free of backticks.
async function gunzipJson(b64){
  const src=Uint8Array.from(atob(b64),c=>c.charCodeAt(0));
  if(typeof DecompressionStream==='function')return JSON.parse(await new Response(new Blob([src]).stream().pipeThrough(new DecompressionStream('gzip'))).text());
  let pos=10;const flg=src[3];
  if(flg&4)pos+=2+(src[pos]|src[pos+1]<<8);
  if(flg&8){while(src[pos++]);}
  if(flg&16){while(src[pos++]);}
  if(flg&2)pos+=2;
  let buf=0,cnt=0;const out=[];
  const bits=n=>{let v=buf;while(cnt<n){v|=src[pos++]<<cnt;cnt+=8}buf=v>>n;cnt-=n;return v&((1<<n)-1)};
  const build=(len,n)=>{const count=new Array(16).fill(0),symbol=new Array(n);for(let i=0;i<n;i++)count[len[i]]++;const offs=[0,0];for(let l=1;l<15;l++)offs[l+1]=offs[l]+count[l];for(let i=0;i<n;i++)if(len[i])symbol[offs[len[i]]++]=i;return{count,symbol}};
  const decode=h=>{let code=0,first=0,index=0;for(let l=1;l<=15;l++){code|=bits(1);const c=h.count[l];if(code-c<first)return h.symbol[index+(code-first)];index+=c;first+=c;first<<=1;code<<=1}throw new Error('bad code')};
  const LB=[3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258],LE=[0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0];
  const DB=[1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577],DE=[0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];
  let fixed=null;
  let last;
  do{
    last=bits(1);const type=bits(2);
    if(type===0){buf=0;cnt=0;const n=src[pos]|src[pos+1]<<8;pos+=4;for(let i=0;i<n;i++)out.push(src[pos++]);continue}
    if(type===3)throw new Error('bad block');
    let lit,dist;
    if(type===1){if(!fixed){const l=[];for(let i=0;i<288;i++)l.push(i<144?8:i<256?9:i<280?7:8);fixed={lit:build(l,288),dist:build(new Array(30).fill(5),30)}}lit=fixed.lit;dist=fixed.dist}
    else{
      const nl=bits(5)+257,nd=bits(5)+1,nc=bits(4)+4,ord=[16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15],cl=new Array(19).fill(0);
      for(let i=0;i<nc;i++)cl[ord[i]]=bits(3);
      const ch=build(cl,19),ls=[];
      while(ls.length<nl+nd){const s=decode(ch);if(s<16)ls.push(s);else{let rep,v=0;if(s===16){v=ls[ls.length-1];rep=3+bits(2)}else if(s===17)rep=3+bits(3);else rep=11+bits(7);while(rep--)ls.push(v)}}
      lit=build(ls.slice(0,nl),nl);dist=build(ls.slice(nl),nd);
    }
    for(;;){
      const s=decode(lit);
      if(s<256)out.push(s);
      else if(s===256)break;
      else{const li=s-257,len=LB[li]+bits(LE[li]),di=decode(dist),d=DB[di]+bits(DE[di]);for(let i=0;i<len;i++)out.push(out[out.length-d])}
    }
  }while(!last);
  return JSON.parse(new TextDecoder().decode(Uint8Array.from(out)));
}

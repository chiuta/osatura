// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
// ---- guided demo: builds the walking person step by step; each step loads that stage of the project (via /api/op, replace-all) ----
const DM={active:false,i:0,orig:null,data:null,tab:'t',seen:new Set(),saveTitle:''};
const DM_BONES=['skeleton'], DM_ANIM=['legs','swing','trunk','face','ground','check'], DM_EXPORT='export';
const DM_KEY='osatura.demo';
function dmMark(on,orig){ try{ if(on){ localStorage.setItem(DM_KEY,'1'); if(orig!==undefined){ try{ localStorage.setItem(DM_KEY+'.orig',JSON.stringify(orig)); }catch(e){ localStorage.removeItem(DM_KEY+'.orig'); } } } else { localStorage.removeItem(DM_KEY); localStorage.removeItem(DM_KEY+'.orig'); localStorage.removeItem(DM_KEY+'.step'); } }catch(e){} }
function dmStored(k){ try{ return localStorage.getItem(DM_KEY+k); }catch(e){ return null; } }
function dmData(){ if(!DM.data)DM.data=JSON.parse($('demo-tour').textContent); return DM.data; }
/*gunzip:begin*/
/*gunzip:end*/
async function dmLoad(){ const D=dmData(); if(D.ready)return; for(const s of D.steps){ if(s.zp){ s.project=await gunzipJson(s.zp); delete s.zp; } } for(const s of D.steps){ if(s.same)s.project=D.steps.find(x=>x.id===s.same).project; } D.ready=true; }
function dmOpen(on){ const b=$('dmp'); const open=on===undefined?b.style.display!=='block':on; b.style.display=open?'block':'none'; if(open&&DM.active)dmFill(); if(open)setTimeout(()=>{ const t=$('dm-run').style.display==='block'?$('dm-next'):$('dm-start'); if(t&&!t.disabled)t.focus(); },0); }
function dmSet(value){ return api('/api/op',{op:'set',path:'',value}); }
async function dmApply(value){ const r=await dmSet(value); if(r&&r.ok===false){ toast((r.errors||['could not load the project']).join('; '),true); return false; }
 S=r; frame=0; $('tl').max=Math.max(0,(S.frames||1)-1); await refresh(true); return true; }
function dmLine(parent,cls,txt){ const s=document.createElement('span'); if(cls)s.className=cls; s.textContent=txt; parent.appendChild(s); }
function dmTerm(){ const st=dmData().steps[DM.i], t=$('dm-term'); t.textContent='';
 if(DM.tab==='s'&&!st.files.length){ const name=st.file||'project.json'; let txt; try{ txt=JSON.stringify(JSON.parse(st.project),null,2); }catch(e){ txt=st.project; }
  const L=txt.split('\n'), MAX=1500, cut=L.length>MAX; if(cut)txt=L.slice(0,MAX).join('\n');
  dmLine(t,'pr','$'); t.appendChild(document.createTextNode(' cat '+name+'\n')); dmLine(t,'out',txt+(cut?'\n  ... ('+(L.length-MAX)+' more lines; '+Math.round(st.project.length/1024)+' KiB in all: use Save or the export buttons for the whole file)\n':'\n')); return; }
 if(DM.tab==='s'&&st.files.length){ st.files.forEach((f,k)=>{ if(k)t.appendChild(document.createTextNode('\n\n')); dmLine(t,'pr','$'); t.appendChild(document.createTextNode(' cat '+f.name+'\n')); dmLine(t,'out',f.text); }); return; }
 st.commands.forEach(c=>{ dmLine(t,'pr','$'); t.appendChild(document.createTextNode(' '+c.cmd+'\n')); if(c.out)dmLine(t,'out',c.out+'\n'); }); }
function dmFill(){ const D=dmData(), st=D.steps[DM.i], n=D.steps.length;
 const tx=DM_TEXT[st.id]; $('dm-num').textContent='Step '+(DM.i+1)+' / '+n; $('dm-title').textContent=tx[0]; $('dm-desc').textContent=tx[1];
 [...$('dm-steps').children].forEach((b,k)=>{ b.setAttribute('aria-selected',k===DM.i); b.classList.toggle('done',DM.seen.has(k)&&k!==DM.i); });
 $('dm-tt').setAttribute('aria-selected',DM.tab==='t'); $('dm-ts').setAttribute('aria-selected',DM.tab==='s'); $('dm-ts').hidden=false;
 $('dm-prev').disabled=DM.i===0; $('dm-next').disabled=DM.i===n-1; dmTerm();
 $('dm-note').textContent=st.id===DM_EXPORT?'Use the GIF, HTML player and Lottie buttons in the header: they export this project, the same files the commands above wrote.':'Real output of '+D.engine.replace(/ \(.*/,'')+'. The project in the editor is that stage, cut from the finished project that osatura demo-person writes.'; }
async function dmGo(i){ const D=dmData(); i=Math.max(0,Math.min(D.steps.length-1,i)); const st=D.steps[i];
 if(!await dmApply(JSON.parse(st.project)))return; DM.i=i; DM.seen.add(i); try{ localStorage.setItem(DM_KEY+'.step',String(i)); }catch(e){}
 $('bones').checked=DM_BONES.includes(st.id); sel=null; show(); bonesUi();
 playing=DM_ANIM.includes(st.id); $('play').textContent=playing?'Pause':'Play'; last=performance.now();
 document.querySelectorAll('header a.btn').forEach(a=>a.classList.toggle('dm-flash',st.id===DM_EXPORT));
 dmFill(); }
function dmBuildSteps(){ $('dm-steps').innerHTML=dmData().steps.map((s,k)=>'<button type="button" role="tab" aria-selected="false" data-k="'+k+'" title="'+DM_TEXT[s.id][0]+'">'+(k+1)+'</button>').join(''); }
async function dmStart(){ try{ await dmLoad(); }catch(e){ toast('could not unpack the demo: '+e.message,true); return; }
 if(typeof COLLAB!=='undefined'&&COLLAB.on){ toast('Leave the collaboration session first: the demo replaces the whole project.',true); return; }
 DM.orig=await api('/api/project'); if(!DM.orig||DM.orig.ok===false){ toast('could not read the open project',true); return; }
 DM.active=true; dmMark(true,DM.orig); DM.seen=new Set(); DM.saveTitle=$('save').title||''; $('save').disabled=true; $('save').title='disabled during the demo: press "Restore my project" first';
 $('dm-intro').style.display='none'; $('dm-run').style.display='block';
 dmBuildSteps();
 await dmGo(0); }
async function dmRestore(){ if(!DM.active)return; if(!DM.orig){ toast('the original project could not be kept in this browser: reopen it with "osatura studio <file>" (nothing was written to disk)',true); return; }
 playing=false; $('play').textContent='Play';
 document.querySelectorAll('header a.btn').forEach(a=>a.classList.remove('dm-flash'));
 if(!await dmApply(DM.orig))return; DM.active=false; dmMark(false); $('save').disabled=false; $('save').title=DM.saveTitle;
 $('dm-run').style.display='none'; $('dm-intro').style.display='block'; toast('your project is back'); }
const DM_TEXT=/*DM_TEXT*/{};
$('demobtn').onclick=e=>{ e.stopPropagation(); dmOpen(); };
$('dm-x').onclick=()=>dmOpen(false);
$('dm-start').onclick=dmStart;
$('dm-prev').onclick=()=>dmGo(DM.i-1); $('dm-next').onclick=()=>dmGo(DM.i+1);
$('dm-steps').onclick=e=>{ const b=e.target.closest('button'); if(b)dmGo(+b.dataset.k); };
$('dm-tt').onclick=()=>{ DM.tab='t'; dmFill(); }; $('dm-ts').onclick=()=>{ DM.tab='s'; dmFill(); };
$('dm-restore').onclick=dmRestore;
window.addEventListener('beforeunload',e=>{ if(DM.active){ e.preventDefault(); e.returnValue=''; } });
$('dmp').addEventListener('keydown',e=>{ if(e.key==='Escape'){ dmOpen(false); $('demobtn').focus(); } });
// the tab was closed or reloaded while the demo was loaded: the server still holds the demo project, so keep Save off and offer the way back
if(dmStored('')==='1'){ (async()=>{
 try{ await dmLoad(); const cur=await api('/api/project');
  if(!cur||cur.name!==dmData().projectName){ dmMark(false); return; }
  const o=dmStored('.orig'); try{ DM.orig=o?JSON.parse(o):null; }catch(e){ DM.orig=null; }
  DM.active=true; DM.saveTitle=$('save').title||''; $('save').disabled=true; $('save').title='disabled during the demo: press "Restore my project" first';
  $('dm-intro').style.display='none'; $('dm-run').style.display='block'; dmBuildSteps();
  DM.i=Math.min(Math.max(0,+(dmStored('.step')||0)),dmData().steps.length-1); DM.seen=new Set([DM.i]); $('dmp').style.display='block'; dmFill();
  toast('A demo project is still loaded in the editor and Save is off. Use "Restore my project".',true);
 }catch(e){ dmMark(false); } })(); }
Object.assign(CMD,{'demo.panel':()=>dmOpen()});

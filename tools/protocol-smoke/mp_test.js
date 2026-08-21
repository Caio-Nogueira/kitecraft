const WebSocket = require('ws');
const zlib = require('zlib');

function varint(v){const out=[];v>>>=0;while(true){if((v&~0x7f)===0){out.push(v);break;}out.push((v&0x7f)|0x80);v>>>=7;}return Buffer.from(out);}
function mcString(s){const b=Buffer.from(s);return Buffer.concat([varint(b.length),b]);}
function payload(id,...parts){return Buffer.concat([varint(id),...parts]);}
function rv(buf,off){let n=0,s=0,i=off;while(true){const b=buf[i++];n|=(b&0x7f)<<s;if(!(b&0x80))break;s+=7;}return [n>>>0,i];}
function f64(v){const b=Buffer.alloc(8);b.writeDoubleBE(v);return b;}
function frame(body,st){
  if(st.threshold===null)return Buffer.concat([varint(body.length),body]);
  const inner=body.length<st.threshold?Buffer.concat([varint(0),body]):Buffer.concat([varint(body.length),zlib.deflateSync(body)]);
  return Buffer.concat([varint(inner.length),inner]);
}

const results=[];
function check(n,ok){results.push(ok);console.log(ok?'PASS':'FAIL','- '+n);if(!ok)process.exitCode=1;}

function mkConn(name,onPacket){
  const st={threshold:null,stage:0};
  const ws=new WebSocket('ws://127.0.0.1:8787/');
  ws.binaryType='nodebuffer';
  ws.on('message',(data)=>{
    let [,off]=rv(data,0);
    let inner=data.subarray(off);
    if(st.threshold!==null){
      const [dl,i2]=rv(inner,0);
      inner=dl===0?inner.subarray(i2):zlib.inflateSync(inner.subarray(i2));
    }
    const [id]=rv(inner,0);
    // login/config driver
    if(st.stage===0&&id===0x03&&st.threshold===null){st.threshold=256;st.stage=1;return;}
    if(st.stage===1&&id===0x02){ws.send(frame(payload(0x03),st));st.stage=2;return;}
    if(st.stage===2&&id===0x0e){ws.send(frame(payload(0x07,varint(1),mcString('minecraft'),mcString('core'),mcString('1.21.4')),st));st.stage=3;return;}
    if(st.stage===3&&id===0x03){ws.send(frame(payload(0x03),st));st.stage=4;onPacket('play-start',null,ws,st);return;}
    if(st.stage>=4)onPacket(id,inner,ws,st);
  });
  ws.on('open',()=>{
    const hs=payload(0x00,varint(769),mcString('t'),(()=>{const p=Buffer.alloc(2);p.writeUInt16BE(25565);return p;})(),varint(2));
    ws.send(Buffer.concat([varint(hs.length),hs]));
    setTimeout(()=>{
      const ls=payload(0x00,mcString(name),Buffer.alloc(16));
      ws.send(Buffer.concat([varint(ls.length),ls]));
    },40);
  });
  return {ws,st};
}

let alicePlay=false;
const seen={bobInfo:false,bobSpawn:false,bobMove:false};

const A=mkConn('Alice',(evt,inner,ws,st)=>{
  if(evt==='play-start'){alicePlay=true;console.log('Alice in play');}
});
const B=mkConn('Bob',(evt,id,ws,st)=>{});

// patch B handler via wrapper: simpler to create B after Alice ready with real handler
process.nextTick(()=>{});

(async()=>{
  while(!alicePlay)await new Promise(r=>setTimeout(r,25));
  await new Promise(r=>setTimeout(r,300));

  const B2=mkConn('Bob',(idOrEvt,inner,ws,st)=>{
    if(idOrEvt==='play-start'){console.log('Bob in play');return;}
    const id=idOrEvt;
    if(id===0x40&&!seen.bobInfo){seen.bobInfo=true;console.log('Bob got player_info');}
    if(id===0x01){
      const [,eidEnd]=rv(inner,1);
      const type=rv(inner,eidEnd+16)[0];
      if(type===147){seen.bobSpawn=true;console.log('Bob got player spawn');}
    }
    if(id===0x20||id===0x77||id===0x30){seen.bobMove=true;console.log('Bob got movement sync id=0x'+id.toString(16));}
  });

  // wait for Bob to enter play
  let waited=0;
  while(!B2.ws || waited<5000){await new Promise(r=>setTimeout(r,50));waited+=50;if(seen.bobInfo||seen.bobSpawn)break;}
  await new Promise(r=>setTimeout(r,800));

  console.log('-- sending Alice movement');
  A.ws.send(frame(payload(0x1c,f64(2.5),f64(-59.0),f64(2.5),Buffer.from([0])),A.st));
  await new Promise(r=>setTimeout(r,900));

  check('Bob received player_info add',seen.bobInfo);
  check('Bob sees Alice entity spawn',seen.bobSpawn);
  check('Bob received Alice movement',seen.bobMove);
  process.exit(process.exitCode||0);
})();

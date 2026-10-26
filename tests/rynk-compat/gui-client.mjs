// Exercise the deployed gui.rmk.rs WASM against the firmware wire types and
// legacy macro adapter. Pass the downloaded JS module and WASM as arguments.
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import assert from 'node:assert/strict';
const [modulePath, wasmPath, peerPath] = process.argv.slice(2);
const wasm = await import(pathToFileURL(resolve(modulePath)));
await wasm.default({module_or_path:readFileSync(wasmPath)});
const peer=spawn(peerPath,[],{stdio:['pipe','pipe','inherit']});
const frames=[]; let pending;
createInterface({input:peer.stdout}).on('line',line=>{
  const bytes=new Uint8Array(Buffer.from(line,'hex'));
  if(pending){const done=pending;pending=undefined;done(bytes);} else frames.push(bytes);
});
let tx=[];
const link={label:'Madula wire compatibility test',
  send:async bytes=>{for(const b of bytes){tx.push(b);if(b===0){if(tx.length>1)peer.stdin.write(Buffer.from(tx).toString('hex')+'\n');tx=[];}}},
  recv:()=>frames.length?Promise.resolve(frames.shift()):new Promise(r=>pending=r)
};
const timeout=setTimeout(()=>{console.error('GUI client test timed out');peer.kill();process.exit(1);},15000);
try {
  if(process.env.TEST_OLD_PROTOCOL) {
    await assert.rejects(wasm.connect(link), /protocol version mismatch/);
    console.log("PASS: original v0.1 mismatch reproduced with deployed GUI WASM");
  } else {
  const client=await wasm.connect(link);
  assert.deepEqual(await client.get_version(),{major:0,minor:2});
  const caps=await client.get_capabilities();
  assert.equal(caps.num_layers,5); assert.equal(caps.num_cols,14);
  assert.equal(caps.max_macros,32); assert.equal(caps.macro_max_size,32);
  assert.equal((await client.get_device_info()).product_name,'Madula RMK');
  const key={Single:{Key:{Hid:'A'}}};
  await client.set_key(0,0,0,key); assert.deepEqual(await client.get_key(0,0,0),key);
  const ops=[{Char:97},{Delay:300},{Tap:{Key:{Hid:'B'}}}];
  await client.write_macro(0,ops); assert.deepEqual(await client.read_macro(0),ops);
  await assert.rejects(client.write_macro(0,['PauseForRelease']));
  assert.deepEqual(await client.read_macro(0),ops);
  await client.get_ble_status();
  console.log('PASS: deployed GUI WASM handshake, capabilities, identity, key read/write, macro read/write, rejection without mutation, BLE status');
  }
} finally {clearTimeout(timeout);peer.kill();}

var Millennium=(()=>{var A=Object.defineProperty,pe=Object.defineProperties,de=Object.getOwnPropertyDescriptor,ge=Object.getOwnPropertyDescriptors,fe=Object.getOwnPropertyNames,B=Object.getOwnPropertySymbols;var K=Object.prototype.hasOwnProperty,ye=Object.prototype.propertyIsEnumerable;var X=(n,e,i)=>e in n?A(n,e,{enumerable:!0,configurable:!0,writable:!0,value:i}):n[e]=i,c=(n,e)=>{for(var i in e||(e={}))K.call(e,i)&&X(n,i,e[i]);if(B)for(var i of B(e))ye.call(e,i)&&X(n,i,e[i]);return n},y=(n,e)=>pe(n,ge(e)),he=n=>A(n,"__esModule",{value:!0});var m=(n,e)=>{for(var i in e)A(n,i,{get:e[i],enumerable:!0})},we=(n,e,i,r)=>{if(e&&typeof e=="object"||typeof e=="function")for(let s of fe(e))!K.call(n,s)&&(i||s!=="default")&&A(n,s,{get:()=>e[s],enumerable:!(r=de(e,s))||r.enumerable});return n};var be=(n=>(e,i)=>n&&n.get(e)||(i=we(he({}),e,1),n&&n.set(e,i),i))(typeof WeakMap!="undefined"?new WeakMap:0);var Nn={};m(Nn,{BaseDirectory:()=>M,app:()=>C,appWindow:()=>T,cli:()=>w,commandLine:()=>w,convertFileSrc:()=>Q,event:()=>F,fileSystem:()=>x,fs:()=>x,globalShortcut:()=>U,http:()=>L,invoke:()=>O,millennium:()=>S,os:()=>I,path:()=>k,platform:()=>E,process:()=>N,shell:()=>W,transformCallback:()=>p,updater:()=>D,window:()=>_,windows:()=>_});var C={};m(C,{getMillenniumVersion:()=>ve,getName:()=>_e,getVersion:()=>xe});var S={};m(S,{convertFileSrc:()=>Q,invoke:()=>O,transformCallback:()=>p});var E={};m(E,{isLinux:()=>q,isMacOS:()=>Y,isWindows:()=>g});function q(){var n,e;return/linux/i.test((e=(n=navigator.userAgentData)==null?void 0:n.platform)!=null?e:navigator.userAgent)}function g(){var n,e;return/win/i.test((e=(n=navigator.userAgentData)==null?void 0:n.platform)!=null?e:navigator.userAgent)}function Y(){var n,e;return/macintosh/i.test((e=(n=navigator.userAgentData)==null?void 0:n.platform)!=null?e:navigator.userAgent)}function Pe(){return window.crypto.getRandomValues(new Uint32Array(1))[0]}function p(n,e=!1){let i=Pe(),r=`_${i}`;return Object.defineProperty(window,r,{value:s=>(e&&delete window[r],n==null?void 0:n(s)),writable:!0,enumerable:!1,configurable:!0}),i}async function O(n,e={}){return new Promise((i,r)=>{let s=p(u=>{i(u),delete window[`_${a}`]},!0),a=p(u=>{r(new Error(u)),delete window[`_${s}`]},!0);window.__MILLENNIUM_IPC__(c({cmd:n,callback:s,error:a},e))})}function Q(n,e="asset"){return g()?`https://${e}.localhost/${n}`:`${e}://${n}`}async function t(n){return O("millennium",n)}async function xe(){return t({__millenniumModule:"App",message:{cmd:"getAppVersion"}})}async function _e(){return t({__millenniumModule:"App",message:{cmd:"getAppName"}})}async function ve(){return t({__millenniumModule:"App",message:{cmd:"getMillenniumVersion"}})}var w={};m(w,{getMatches:()=>Me});async function Me(){return await t({__millenniumModule:"Cli",message:{cmd:"cliMatches"}})}var F={};m(F,{emit:()=>b,listen:()=>P,once:()=>v});async function Z(n,e){return t({__millenniumModule:"Event",message:{cmd:"unlisten",event:n,eventId:e}})}async function b(n,e,i){await t({__millenniumModule:"Event",message:{cmd:"emit",event:n,windowLabel:e,payload:typeof i=="string"?i:JSON.stringify(i)}})}async function P(n,e,i){let r=await t({__millenniumModule:"Event",message:{cmd:"listen",event:n,windowLabel:e,handler:p(i)}});return async()=>Z(n,r)}async function v(n,e,i){return P(n,e,r=>{i(r),Z(n,r.id).catch(()=>{})})}var x={};m(x,{BaseDirectory:()=>M,copyFile:()=>Ce,exists:()=>Le,mkdir:()=>Oe,readBinaryFile:()=>ne,readFile:()=>Te,readTextFile:()=>ee,readdir:()=>Ee,removeFile:()=>Fe,rename:()=>Ue,rmdir:()=>Se,writeBinaryFile:()=>V,writeFile:()=>Ae,writeTextFile:()=>ie});var M=(o=>(o[o.AUDIO=1]="AUDIO",o[o.CACHE=2]="CACHE",o[o.CONFIG=3]="CONFIG",o[o.DATA=4]="DATA",o[o.LOCALDATA=5]="LOCALDATA",o[o.DESKTOP=6]="DESKTOP",o[o.DOCUMENT=7]="DOCUMENT",o[o.DOWNLOAD=8]="DOWNLOAD",o[o.EXECUTABLE=9]="EXECUTABLE",o[o.FONT=10]="FONT",o[o.HOME=11]="HOME",o[o.PICTURES=12]="PICTURES",o[o.PUBLIC=13]="PUBLIC",o[o.RUNTIME=14]="RUNTIME",o[o.TEMPLATES=15]="TEMPLATES",o[o.VIDEOS=16]="VIDEOS",o[o.RESOURCE=17]="RESOURCE",o[o.APP=18]="APP",o[o.LOG=19]="LOG",o))(M||{});async function ee(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"readTextFile",path:n,options:e}})}async function ne(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"readFile",path:n,options:e}})}async function Te(n,e,i={}){switch(typeof e!="string"&&(i=e,e="binary"),e){case"utf8":case"utf-8":return await ee(n,i);case"binary":return await ne(n,i);default:throw new Error(`Unsupported encoding: '${e}'. Consider reading as binary and using \`TextDecoder\`.`)}}async function ie(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"writeTextFile",path:n.path,contents:Array.from(new TextEncoder().encode(n.contents)),options:e}})}async function V(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"writeFile",path:n.path,contents:Array.from(n.contents),options:e}})}async function Ae(n,e,i){if(typeof e=="string")return await ie({path:n,contents:e},i);if(e instanceof ArrayBuffer||e instanceof SharedArrayBuffer)return await V({path:n,contents:Array.from(new Uint8Array(e))},i);if(e instanceof DataView||e instanceof Int8Array||e instanceof Uint8Array||e instanceof Uint8ClampedArray||e instanceof Int16Array||e instanceof Uint16Array||e instanceof Int32Array||e instanceof Uint32Array||e instanceof Float32Array||typeof Float64Array<"u"&&e instanceof Float64Array||typeof BigInt64Array<"u"&&e instanceof BigInt64Array||typeof BigUint64Array<"u"&&e instanceof BigUint64Array)return await V({path:n,contents:Array.from(new Uint8Array(e.buffer.slice(e.byteOffset,e.byteOffset+e.byteLength)))},i);throw new Error(`Unsupported contents type: '${{}.toString.call(e)}'.`)}async function Ee(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"readDir",path:n,options:e}})}async function Oe(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"createDir",path:n,options:e}})}async function Se(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"removeDir",path:n,options:e}})}async function Ce(n,e,i={}){return await t({__millenniumModule:"Fs",message:{cmd:"copyFile",source:n,destination:e,options:i}})}async function Fe(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"removeFile",path:n,options:e}})}async function Ue(n,e,i={}){return await t({__millenniumModule:"Fs",message:{cmd:"rename",oldPath:n,newPath:e,options:i}})}async function Le(n,e={}){return await t({__millenniumModule:"Fs",message:{cmd:"exists",path:n,options:e}})}var U={};m(U,{isRegistered:()=>Ne,register:()=>Ie,registerAll:()=>ke,unregister:()=>Re,unregisterAll:()=>We});async function Ie(n,e){return await t({__millenniumModule:"GlobalShortcut",message:{cmd:"register",shortcut:n,handler:p(e)}})}async function ke(n,e){return await t({__millenniumModule:"GlobalShortcut",message:{cmd:"registerAll",shortcuts:n,handler:p(e)}})}async function Ne(n){return await t({__millenniumModule:"GlobalShortcut",message:{cmd:"isRegistered",shortcut:n}})}async function Re(n){return await t({__millenniumModule:"GlobalShortcut",message:{cmd:"unregister",shortcut:n}})}async function We(){return await t({__millenniumModule:"GlobalShortcut",message:{cmd:"unregisterAll"}})}var L={};m(L,{Body:()=>h,Response:()=>$,ResponseType:()=>te,createClient:()=>se,fetch:()=>De});var te=(r=>(r[r.JSON=1]="JSON",r[r.TEXT=2]="TEXT",r[r.BINARY=3]="BINARY",r))(te||{}),h=class{constructor(e,i){this.type=e;this.payload=i}static form(e){let i={};for(let r in e){let s=e[r];i[r]=typeof s=="string"?s:Array.from(s)}return new h("Form",i)}static json(e){return new h("Json",e)}static text(e){return new h("Text",e)}static binary(e){return new h("Bytes",Array.from(e))}},$=class{constructor(e){this.url=e.url,this.status=e.status,this.ok=e.status>=200&&e.status<300,this.headers=e.headers,this.rawHeaders=e.rawHeaders,this.data=e.data}},re=class{constructor(e){this.id=e}async drop(){return t({__millenniumModule:"Http",message:{cmd:"dropClient",client:this.id}})}async request(e){let i=!e.responseType||e.responseType===1;i&&(e.responseType=2);let r=await t({__millenniumModule:"Http",message:{cmd:"httpRequest",client:this.id,options:e}}),s=new $(r);if(i)try{s.data=JSON.parse(s.data)}catch(a){if(s.ok&&s.data==="")s.data={};else if(s.ok)throw new Error(`Failed to parse response body (\`${s.data}\`): ${a}
Try setting the responseType to a different type if the API doesn't return JSON.`)}return s}async get(e,i){return this.request(y(c({},i),{method:"GET",url:e}))}async post(e,i,r){return this.request(y(c({},r),{method:"POST",body:i,url:e}))}async put(e,i,r){return this.request(y(c({},r),{method:"PUT",body:i,url:e}))}async patch(e,i){return this.request(y(c({},i),{method:"PATCH",url:e}))}async delete(e,i){return this.request(y(c({},i),{method:"DELETE",url:e}))}};async function se(n){let e=await t({__millenniumModule:"Http",message:{cmd:"createClient",options:n}});return new re(e)}var G=null;async function De(n,e){var i;return G||(G=await se()),G.request(y(c({},e),{url:n,method:(i=e==null?void 0:e.method)!=null?i:"GET"}))}var I={};m(I,{arch:()=>Ve,eol:()=>ze,platform:()=>He,tmpdir:()=>Ge,type:()=>Ye,version:()=>qe});var ze=g()?`\r
`:`
`;async function He(){return t({__millenniumModule:"Os",message:{cmd:"platform"}})}async function qe(){return t({__millenniumModule:"Os",message:{cmd:"version"}})}async function Ye(){return t({__millenniumModule:"Os",message:{cmd:"type"}})}async function Ve(){return t({__millenniumModule:"Os",message:{cmd:"arch"}})}async function Ge(){return t({__millenniumModule:"Os",message:{cmd:"tempdir"}})}var k={};m(k,{appDir:()=>$e,audioDir:()=>je,basename:()=>bn,cacheDir:()=>Je,configDir:()=>Be,dataDir:()=>Xe,delimiter:()=>dn,desktopDir:()=>Ke,dirname:()=>hn,documentsDir:()=>Qe,downloadsDir:()=>Ze,executableDir:()=>en,extname:()=>wn,fontDir:()=>nn,homeDir:()=>tn,isAbsolute:()=>Pn,join:()=>yn,localDataDir:()=>rn,logDir:()=>cn,normalize:()=>fn,pictureDir:()=>sn,publicDir:()=>on,resolve:()=>gn,resourceDir:()=>an,runtimeDir:()=>ln,sep:()=>pn,templateDir:()=>un,videosDir:()=>mn});var l=async n=>await t({__millenniumModule:"Path",message:{cmd:"resolvePath",path:"",directory:n}});async function $e(){return l(18)}async function je(){return l(1)}async function Je(){return l(2)}async function Be(){return l(3)}async function Xe(){return l(4)}async function Ke(){return l(6)}async function Qe(){return l(7)}async function Ze(){return l(8)}async function en(){return l(9)}async function nn(){return l(10)}async function tn(){return l(11)}async function rn(){return l(5)}async function sn(){return l(12)}async function on(){return l(13)}async function an(){return l(17)}async function ln(){return l(14)}async function un(){return l(15)}async function mn(){return l(16)}async function cn(){return l(19)}var pn=g()?"\\":"/",dn=g()?";":":";async function gn(...n){return t({__millenniumModule:"Path",message:{cmd:"resolve",paths:n}})}async function fn(n){return t({__millenniumModule:"Path",message:{cmd:"normalize",path:n}})}async function yn(...n){return t({__millenniumModule:"Path",message:{cmd:"join",paths:n}})}async function hn(n){return t({__millenniumModule:"Path",message:{cmd:"dirname",path:n}})}async function wn(n){return t({__millenniumModule:"Path",message:{cmd:"extname",path:n}})}async function bn(n,e){return t({__millenniumModule:"Path",message:{cmd:"basename",path:n,ext:e}})}async function Pn(n){return t({__millenniumModule:"Path",message:{cmd:"isAbsolute",path:n}})}var N={};m(N,{exit:()=>xn,relaunch:()=>_n});async function xn(n=0){await t({__millenniumModule:"Process",message:{cmd:"exit",exitCode:n}})}async function _n(){await t({__millenniumModule:"Process",message:{cmd:"relaunch"}})}var W={};m(W,{Command:()=>d,exec:()=>Tn,execSidecar:()=>An,open:()=>Mn,showItemInFolder:()=>Sn,spawn:()=>En,spawnSidecar:()=>On});async function vn(n,e,i=[],r){return typeof i=="object"&&Object.freeze(i),t({__millenniumModule:"Shell",message:{cmd:"execute",program:e,args:i,options:r,onEventFn:p(n)}})}var R=class{constructor(){this.eventListeners=new Map}addEventListener(e,i){return this.eventListeners.has(e)?this.eventListeners.get(e).push(i):this.eventListeners.set(e,[i]),this}on(e,i){return this.addEventListener(e,i)}emit(e,i){return this.eventListeners.has(e)&&this.eventListeners.get(e).forEach(r=>r(i)),this}},oe=class{constructor(e){this.pid=e}async write(e){return await t({__millenniumModule:"Shell",message:{cmd:"stdinWrite",pid:this.pid,buffer:typeof e=="string"?e:Array.from(e)}})}async kill(){return await t({__millenniumModule:"Shell",message:{cmd:"killChild",pid:this.pid}})}},d=class extends R{constructor(e,i=[],r={}){super();this.program=e;this.stdout=new R;this.stderr=new R;this.args=typeof i=="string"?[i]:i,this.options=r}static sidecar(e,i=[],r){let s=new d(e,i,r);return s.options.sidecar=!0,s}async spawn(){let i=await vn(({event:r,payload:s})=>{switch(r){case"Error":this.emit("error",s);break;case"Terminated":this.emit("close",s);break;case"Stdout":this.stdout.emit("data",s);break;case"Stderr":this.stderr.emit("data",s);break}},this.program,this.args,this.options);return new oe(i)}execute(){return new Promise((e,i)=>{this.on("error",i);let r=[],s=[];this.stdout.on("data",a=>r.push(a)),this.stderr.on("data",a=>s.push(a)),this.on("close",a=>{e({code:a.code,signal:a.signal,stdout:r.join(`
`),stderr:s.join(`
`)})}),this.spawn().catch(i)})}};async function Mn(n,e){return await t({__millenniumModule:"Shell",message:{cmd:"open",path:n,with:e}})}async function Tn(n,e=[],i={}){return await new d(n,e,i).execute()}async function An(n,e=[],i={}){return await d.sidecar(n,e,i).execute()}async function En(n,e=[],i={}){return await new d(n,e,i).spawn()}async function On(n,e=[],i={}){return await d.sidecar(n,e,i).spawn()}async function Sn(n){if(g())await new d("explorer",["/select,",n]).execute();else if(q())await new d("dbus-send",["--session","--print-reply","--dest=org.freedesktop.FileManager1","--type=method_call","/org/freedesktop/FileManager1","org.freedesktop.FileManager1.ShowItems",`array:string:"file://${n}"`,'string:""']).execute();else if(Y())await new d("open",["-R",n]).execute();else throw new Error("Unsupported platform")}var D={};m(D,{checkForUpdates:()=>Fn,installUpdate:()=>Cn});function Cn(){let n;function e(){n==null||n(),n=void 0}return new Promise((i,r)=>{function s(a){if(a.error)return e(),r(a.error);if(a.status==="DONE")return e(),i()}P("millennium://update-status",null,a=>{s(a==null?void 0:a.payload)}).then(a=>n=a).catch(a=>{throw e(),a}),b("millennium://update-install").catch(a=>{throw e(),a})})}function Fn(){let n;function e(){n==null||n(),n=void 0}return new Promise((i,r)=>{function s(u){return e(),i({manifest:u,shouldUpdate:!0})}function a(u){if(u.error)return e(),r(u.error);if(u.status==="UPTODATE")return e(),i({shouldUpdate:!1})}v("millennium://update-available",null,u=>{s(u==null?void 0:u.payload)}).catch(u=>{throw e(),u}),b("millennium://update").catch(u=>{throw e(),u})})}var _={};m(_,{LogicalPosition:()=>J,LogicalSize:()=>j,PhysicalPosition:()=>H,PhysicalSize:()=>z,UserAttentionType:()=>le,WebviewWindow:()=>f,appWindow:()=>T,availableMonitors:()=>kn,currentMonitor:()=>Ln,getAllWindows:()=>ue,getCurrentWindow:()=>Un,primaryMonitor:()=>In});var j=class{constructor(e,i){this.width=e;this.height=i;this.TYPE="logical"}},z=class{constructor(e,i){this.width=e;this.height=i;this.TYPE="physical"}toLogical(e){return new j(this.width/e,this.height/e)}},J=class{constructor(e,i){this.x=e;this.y=i;this.TYPE="logical"}},H=class{constructor(e,i){this.x=e;this.y=i;this.TYPE="physical"}toLogical(e){return new J(this.x/e,this.y/e)}},le=(i=>(i[i.CRITICAL=1]="CRITICAL",i[i.INFORMATIONAL=2]="INFORMATIONAL",i))(le||{});function Un(){return new f(window.__MILLENNIUM_METADATA__.__currentWindow.label,{skip:!0})}function ue(){return window.__MILLENNIUM_METADATA__.__windows.map(({label:n})=>new f(n,{skip:!0}))}var ae=["millennium://created","millennium://error"],me=class{constructor(e){this.label=e;this.listeners=new Map}async listen(e,i){if(this.handleMillenniumEvent(e,i)){let r=this.listeners.get(e);return r==null||r.splice(r.indexOf(i),1),()=>{}}return await P(e,this.label,i)}async once(e,i){if(this.handleMillenniumEvent(e,i)){let r=this.listeners.get(e);return r.splice(r.indexOf(i),1),()=>{}}return await v(e,this.label,i)}async emit(e,i){var r;if(ae.includes(e)){for(let s of(r=this.listeners.get(e))!=null?r:[])s({event:e,id:-1,windowLabel:this.label,payload:i});return}return await b(e,this.label,i)}handleMillenniumEvent(e,i){return ae.includes(e)?(this.listeners.has(e)?this.listeners.get(e).push(i):this.listeners.set(e,[i]),!0):!1}},ce=class extends me{async _manage(e,i){return await t({__millenniumModule:"Window",message:{cmd:"manage",data:{label:this.label,cmd:c({type:e},i!==void 0?{payload:i}:{})}}})}async scaleFactor(){return await this._manage("scaleFactor")}async innerPosition(){return await this._manage("position").then(({x:e,y:i})=>new H(e,i))}async outerPosition(){return await this._manage("outerPosition").then(({x:e,y:i})=>new H(e,i))}async innerSize(){return await this._manage("innerSize").then(({width:e,height:i})=>new z(e,i))}async outerSize(){return await this._manage("outerSize").then(({width:e,height:i})=>new z(e,i))}async isFullscreen(){return await this._manage("isFullscreen")}async isMaximized(){return await this._manage("isMaximized")}async isDecorated(){return await this._manage("isDecorated")}async isResizable(){return await this._manage("isResizable")}async isVisible(){return await this._manage("isVisible")}async center(){return await this._manage("center")}async requestUserAttention(e=null){return await this._manage("requestUserAttention",e===1?{type:"Critical"}:e===2?{type:"Informational"}:null)}async setResizable(e){return await this._manage("setResizable",e)}async setTitle(e){return await this._manage("setTitle",e)}async maximize(){return await this._manage("maximize")}async unmaximize(){return await this._manage("unmaximize")}async toggleMaximized(){return await this._manage("toggleMaximized")}async minimize(){return await this._manage("minimize")}async unminimize(){return await this._manage("unminimize")}async show(){return await this._manage("show")}async hide(){return await this._manage("hide")}async close(){return await this._manage("close")}async setDecorations(e){return await this._manage("setDecorations",e)}async setAlwaysOnTop(e){return await this._manage("setAlwaysOnTop",e)}async setSize(e){if(!e||e.TYPE!=="logical"&&e.TYPE!=="physical")throw new Error("Invalid size! Must be an instance of `LogicalSize` or `PhysicalSize`.");return await this._manage("setSize",{type:e.TYPE,data:{width:e.width,height:e.height}})}async setMinimumSize(e=null){if(!e||e.TYPE!=="logical"&&e.TYPE!=="physical")throw new Error("Invalid size! Must be an instance of `LogicalSize` or `PhysicalSize`.");return await this._manage("setMinSize",e===null?null:{type:e.TYPE,data:{width:e.width,height:e.height}})}async setMaximumSize(e=null){if(!e||e.TYPE!=="logical"&&e.TYPE!=="physical")throw new Error("Invalid size! Must be an instance of `LogicalSize` or `PhysicalSize`.");return await this._manage("setMaxSize",e===null?null:{type:e.TYPE,data:{width:e.width,height:e.height}})}async setPosition(e){if(!e||e.TYPE!=="logical"&&e.TYPE!=="physical")throw new Error("Invalid position! Must be an instance of `LogicalPosition` or `PhysicalPosition`.");return await this._manage("setPosition",{type:e.TYPE,data:{x:e.x,y:e.y}})}async setFullscreen(e){return await this._manage("setFullscreen",e)}async focus(){return await this._manage("setFocus")}async setIcon(e){return await this._manage("setIcon",{icon:typeof e=="string"?e:Array.from(e)})}async setShowInTaskbar(e=!0){return await this._manage("setSkipTaskbar",e)}async startDragging(){return await this._manage("startDragging")}},f=class extends ce{constructor(e,i={}){super(e);i.skip||t({__millenniumModule:"Window",message:{cmd:"createWebview",data:{options:c({label:e},i)}}}).then(async()=>this.emit("millennium://created")).catch(async r=>this.emit("millennium://error",r))}static getByLabel(e){return ue().some(i=>i.label===e)?new f(e,{skip:!0}):null}},T;"__MILLENNIUM_METADATA__"in window?T=new f(window.__MILLENNIUM_METADATA__.__currentWindow.label,{skip:!0}):(console.warn(`Could not find __MILLENNIUM_METADATA__. The "appWindow" value will reference the window with the "main" label.
This is not an issue if you are running this frontend in a browser instead of a Millennium window.`),T=new f("main",{skip:!0}));async function Ln(){return await t({__millenniumModule:"Window",message:{cmd:"manage",data:{cmd:{type:"currentMonitor"}}}})}async function In(){return t({__millenniumModule:"Window",message:{cmd:"manage",data:{cmd:{type:"primaryMonitor"}}}})}async function kn(){return await t({__millenniumModule:"Window",message:{cmd:"manage",data:{cmd:{type:"availableMonitors"}}}})}return be(Nn);})();
;Object.defineProperty(window,"Millennium",{value:Millennium,writable:false,configurable:false,enumerable:true});function _DF(e){const d=Object.getOwnPropertyNames(e);for(const g of d)if(typeof e[g]=="object")_DF(e[g]);Object.freeze(e)}_DF(window.Millennium);
